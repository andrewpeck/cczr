use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{Color, colorize};
use crate::plugin::{Plugin, PluginResult, PluginType};

// LEVEL: [Module Code-Num] message
// Handles INFO, WARNING, CRITICAL WARNING, ERROR
static RE_MSG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(CRITICAL WARNING|WARNING|ERROR|INFO): (\[[^\]]+\]) (.*)$").unwrap()
});

static RE_FILE_LOC: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[/[^\]\n]+\]").unwrap());
static RE_STRING: Lazy<Regex> = Lazy::new(|| Regex::new(r#""[^"]*""#).unwrap());
static RE_ABS_PATH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(^|[\s\['"])(/[^\s,\]'"]+)"#).unwrap());
static RE_TIMING_FIELD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:WNS|TNS|WHS|THS)=\s*(N/A|[-+]?\d+(?:\.\d+)?)").unwrap());
static RE_COUNTED_SEVERITY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d+ (?:Critical Warnings|Warnings|Errors|Infos)\b").unwrap());
static RE_VIVADO_NUMBER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(^|[^A-Za-z0-9_.])([-+]?\d+(?:\.\d+)?)([^A-Za-z0-9_.]|$)").unwrap());

// Timing + memory stats line emitted after most operations:
//   Description: Time (s): cpu = HH:MM:SS.ss ; elapsed = HH:MM:SS.ss . Memory (MB): peak = N ; gain = N ; free physical = N ; free virtual = N
static RE_TIMING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(.*?)[.:]\s*Time \(s\): cpu = ([\d:.]+) ; elapsed = ([\d:.]+) \. Memory \(MB\): peak = ([\d.]+) ; gain = ([\d.]+) ; free physical = (\d+) ; free virtual = (\d+)$"
    ).unwrap()
});

// Restored from archive | CPU: N secs | Memory: N MB |
static RE_ARCHIVE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(Restored from archive) \| CPU: ([\d.]+) secs \| Memory: ([\d.]+) MB \|$")
        .unwrap()
});

static RE_SUMMARY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\d+ Infos), (\d+ Warnings), (\d+ Critical Warnings) and (\d+ Errors) encountered\.$",
    )
    .unwrap()
});

static RE_STATUS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\s*)(.+?) (completed successfully|failed)$").unwrap());

static RE_LABELED_DETAIL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(New|Previous|Resolution):\s*(.*)$").unwrap());

// Header comment lines  (#--- separator, # Key : Value, # plain)
static RE_COMMENT_SEP: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(#-{3,})$").unwrap());
static RE_COMMENT_KV: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#\s+([^:]+?)\s*:\s+(.*)$").unwrap());
static RE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#(.*)$").unwrap());

// Tcl commands – lines that look like bare Tcl (no prefix, not a timing line,
// not a comment).  We match a leading word that is a known Vivado Tcl command.
static RE_TCL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(start_gui|end_gui|open_project|close_project|open_run|close_run|open_impl|close_impl|open_synth|close_synth|launch_runs|wait_on_run|create_ip_run|refresh_design|update_compile_order|set_property|get_property|get_nets|get_cells|get_pins|get_ports|get_clocks|get_files|get_fileset|report_timing_summary|report_bus_skew|report_utilization|report_power|report_drc|write_bitstream|write_checkpoint|read_checkpoint|link_design|synth_design|opt_design|place_design|phys_opt_design|route_design|impl_design)(\s.*)?$"
    ).unwrap()
});

fn push_span(spans: &mut Vec<(usize, usize, Color)>, start: usize, end: usize, color: Color) {
    if start < end {
        spans.push((start, end, color));
    }
}

fn counted_severity_color(text: &str) -> Color {
    let is_zero = text.starts_with("0 ");
    if text.ends_with("Infos") {
        Color::Info
    } else if is_zero {
        Color::GoodWord
    } else if text.ends_with("Critical Warnings") || text.ends_with("Errors") {
        Color::Error
    } else {
        Color::Warning
    }
}

fn timing_value_color(value: &str) -> Color {
    if value == "N/A" {
        return Color::Unknown;
    }
    if value.starts_with('-') {
        return Color::BadWord;
    }
    match value.parse::<f64>() {
        Ok(v) if v > 0.0 => Color::GoodWord,
        _ => Color::Numbers,
    }
}

fn command_status_subject_color(subject: &str) -> Color {
    if subject.contains('_') {
        Color::Keyword
    } else {
        Color::Default
    }
}

fn render_spans(input: &str, mut spans: Vec<(usize, usize, Color)>) -> String {
    if spans.is_empty() {
        return input.to_owned();
    }

    spans.sort_by_key(|&(s, e, _)| (s, std::cmp::Reverse(e)));

    let mut out = String::with_capacity(input.len() * 2);
    let mut cursor = 0usize;
    for (s, e, color) in spans {
        if s < cursor {
            continue;
        }
        out.push_str(&input[cursor..s]);
        out.push_str(&colorize(color, &input[s..e]));
        cursor = e;
    }
    out.push_str(&input[cursor..]);
    out
}

fn colorize_vivado_message(input: &str) -> String {
    let mut spans: Vec<(usize, usize, Color)> = Vec::new();

    for m in RE_STRING.find_iter(input) {
        push_span(&mut spans, m.start(), m.end(), Color::String);
    }
    for m in RE_FILE_LOC.find_iter(input) {
        push_span(&mut spans, m.start(), m.end(), Color::Dir);
    }
    for cap in RE_ABS_PATH.captures_iter(input) {
        if let Some(path) = cap.get(2) {
            push_span(&mut spans, path.start(), path.end(), Color::Dir);
        }
    }
    for cap in RE_TIMING_FIELD.captures_iter(input) {
        if let Some(value) = cap.get(1) {
            push_span(
                &mut spans,
                value.start(),
                value.end(),
                timing_value_color(value.as_str()),
            );
        }
    }
    for m in RE_COUNTED_SEVERITY.find_iter(input) {
        push_span(
            &mut spans,
            m.start(),
            m.end(),
            counted_severity_color(m.as_str()),
        );
    }
    for cap in RE_VIVADO_NUMBER.captures_iter(input) {
        if let Some(value) = cap.get(2) {
            push_span(&mut spans, value.start(), value.end(), Color::Numbers);
        }
    }

    render_spans(input, spans)
}

pub struct Vivado;

impl Plugin for Vivado {
    fn name(&self) -> &'static str {
        "vivado"
    }
    fn kind(&self) -> PluginType {
        PluginType::Full
    }

    fn process(&self, line: &str) -> PluginResult {
        // ── severity message ─────────────────────────────────────────────────
        if let Some(caps) = RE_MSG.captures(line) {
            let level = caps.get(1).map_or("", |m| m.as_str());
            let tag = caps.get(2).map_or("", |m| m.as_str());
            let msg = caps.get(3).map_or("", |m| m.as_str());

            let (level_color, tag_color) = match level {
                "INFO" => (Color::Info, Color::Ident),
                "WARNING" => (Color::Warning, Color::Ident),
                "CRITICAL WARNING" => (Color::Error, Color::Error),
                "ERROR" => (Color::Error, Color::Error),
                _ => (Color::Default, Color::Default),
            };

            let out = format!(
                "{}: {} {}",
                colorize(level_color, level),
                colorize(tag_color, tag),
                colorize_vivado_message(msg),
            );
            return PluginResult::Matched(out);
        }

        // ── timing / memory stats ─────────────────────────────────────────────
        if let Some(caps) = RE_TIMING.captures(line) {
            let desc = caps.get(1).map_or("", |m| m.as_str());
            let cpu = caps.get(2).map_or("", |m| m.as_str());
            let elapsed = caps.get(3).map_or("", |m| m.as_str());
            let peak = caps.get(4).map_or("", |m| m.as_str());
            let gain = caps.get(5).map_or("", |m| m.as_str());
            let free_phy = caps.get(6).map_or("", |m| m.as_str());
            let free_vir = caps.get(7).map_or("", |m| m.as_str());

            let out = format!(
                "{}: Time (s): cpu = {} ; elapsed = {} . Memory (MB): peak = {} ; gain = {} ; free physical = {} ; free virtual = {}",
                colorize(Color::Default, desc),
                colorize(Color::GetTime, cpu),
                colorize(Color::GetTime, elapsed),
                colorize(Color::Size, peak),
                colorize(Color::Numbers, gain),
                colorize(Color::Numbers, free_phy),
                colorize(Color::Numbers, free_vir),
            );
            return PluginResult::Matched(out);
        }

        // ── restored from archive ─────────────────────────────────────────────
        if let Some(caps) = RE_ARCHIVE.captures(line) {
            let label = caps.get(1).map_or("", |m| m.as_str());
            let cpu = caps.get(2).map_or("", |m| m.as_str());
            let mem = caps.get(3).map_or("", |m| m.as_str());
            let out = format!(
                "{} | CPU: {} secs | Memory: {} MB |",
                colorize(Color::GoodWord, label),
                colorize(Color::GetTime, cpu),
                colorize(Color::Size, mem),
            );
            return PluginResult::Matched(out);
        }

        // ── cumulative severity summary ─────────────────────────────────────
        if let Some(caps) = RE_SUMMARY.captures(line) {
            let infos = caps.get(1).map_or("", |m| m.as_str());
            let warnings = caps.get(2).map_or("", |m| m.as_str());
            let critical = caps.get(3).map_or("", |m| m.as_str());
            let errors = caps.get(4).map_or("", |m| m.as_str());
            let out = format!(
                "{}, {}, {} and {} encountered.",
                colorize(counted_severity_color(infos), infos),
                colorize(counted_severity_color(warnings), warnings),
                colorize(counted_severity_color(critical), critical),
                colorize(counted_severity_color(errors), errors),
            );
            return PluginResult::Matched(out);
        }

        // ── command status lines ─────────────────────────────────────────────
        if let Some(caps) = RE_STATUS.captures(line) {
            let indent = caps.get(1).map_or("", |m| m.as_str());
            let subject = caps.get(2).map_or("", |m| m.as_str());
            let status = caps.get(3).map_or("", |m| m.as_str());
            let status_color = if status == "failed" {
                Color::Error
            } else {
                Color::GoodWord
            };
            let out = format!(
                "{}{} {}",
                indent,
                colorize(command_status_subject_color(subject), subject),
                colorize(status_color, status),
            );
            return PluginResult::Matched(out);
        }

        // ── Vivado message continuation details ──────────────────────────────
        if let Some(caps) = RE_LABELED_DETAIL.captures(line) {
            let label = caps.get(1).map_or("", |m| m.as_str());
            let detail = caps.get(2).map_or("", |m| m.as_str());
            let out = format!(
                "{}: {}",
                colorize(Color::Field, label),
                colorize_vivado_message(detail),
            );
            return PluginResult::Matched(out);
        }

        // ── header comment separator  #------- ────────────────────────────────
        if RE_COMMENT_SEP.is_match(line) {
            return PluginResult::Matched(colorize(Color::Debug, line));
        }

        // ── header comment key-value  # Key   : value ─────────────────────────
        if let Some(caps) = RE_COMMENT_KV.captures(line) {
            let key = caps.get(1).map_or("", |m| m.as_str());
            let val = caps.get(2).map_or("", |m| m.as_str());
            let out = format!(
                "# {}: {}",
                colorize(Color::Field, key),
                colorize(Color::Default, val),
            );
            return PluginResult::Matched(out);
        }

        // ── generic comment line ──────────────────────────────────────────────
        if let Some(caps) = RE_COMMENT.captures(line) {
            let body = caps.get(1).map_or("", |m| m.as_str());
            let out = format!("#{}", colorize(Color::Debug, body));
            return PluginResult::Matched(out);
        }

        // ── Tcl command ───────────────────────────────────────────────────────
        if let Some(caps) = RE_TCL.captures(line) {
            let cmd = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let out = format!(
                "{}{}",
                colorize(Color::Keyword, cmd),
                colorize(Color::Default, args)
            );
            return PluginResult::Matched(out);
        }

        PluginResult::NoMatch
    }
}
