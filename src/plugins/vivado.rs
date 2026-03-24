use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};
use crate::wordcolor::colorize_words;

// LEVEL: [Module Code-Num] message
// Handles INFO, WARNING, CRITICAL WARNING, ERROR
static RE_MSG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(CRITICAL WARNING|WARNING|ERROR|INFO): (\[[\w.]+ \d+-\d+\]) (.*)$").unwrap()
});

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

// Header comment lines  (#--- separator, # Key : Value, # plain)
static RE_COMMENT_SEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(#-{3,})$").unwrap()
});
static RE_COMMENT_KV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^#\s+([^:]+?)\s*:\s+(.*)$").unwrap()
});
static RE_COMMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^#(.*)$").unwrap()
});

// Tcl commands – lines that look like bare Tcl (no prefix, not a timing line,
// not a comment).  We match a leading word that is a known Vivado Tcl command.
static RE_TCL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(start_gui|end_gui|open_project|close_project|open_run|close_run|open_impl|close_impl|open_synth|close_synth|launch_runs|wait_on_run|create_ip_run|refresh_design|update_compile_order|set_property|get_property|get_nets|get_cells|get_pins|get_ports|get_clocks|get_files|get_fileset|report_timing_summary|report_bus_skew|report_utilization|report_power|report_drc|write_bitstream|write_checkpoint|read_checkpoint|synth_design|opt_design|place_design|phys_opt_design|route_design|impl_design)(\s.*)?$"
    ).unwrap()
});

pub struct Vivado;

impl Plugin for Vivado {
    fn name(&self) -> &'static str { "vivado" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        // ── severity message ─────────────────────────────────────────────────
        if let Some(caps) = RE_MSG.captures(line) {
            let level = caps.get(1).map_or("", |m| m.as_str());
            let tag   = caps.get(2).map_or("", |m| m.as_str());
            let msg   = caps.get(3).map_or("", |m| m.as_str());

            let (level_color, tag_color) = match level {
                "INFO"             => (Color::Info,      Color::Ident),
                "WARNING"          => (Color::Warning,  Color::Ident),
                "CRITICAL WARNING" => (Color::Error,    Color::Error),
                "ERROR"            => (Color::Error,    Color::Error),
                _                  => (Color::Default,  Color::Default),
            };

            let out = format!(
                "{}: {} {}",
                colorize(level_color, level),
                colorize(tag_color, tag),
                colorize_words(msg),
            );
            return PluginResult::Matched(out);
        }

        // ── timing / memory stats ─────────────────────────────────────────────
        if let Some(caps) = RE_TIMING.captures(line) {
            let desc     = caps.get(1).map_or("", |m| m.as_str());
            let cpu      = caps.get(2).map_or("", |m| m.as_str());
            let elapsed  = caps.get(3).map_or("", |m| m.as_str());
            let peak     = caps.get(4).map_or("", |m| m.as_str());
            let gain     = caps.get(5).map_or("", |m| m.as_str());
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
            let label  = caps.get(1).map_or("", |m| m.as_str());
            let cpu    = caps.get(2).map_or("", |m| m.as_str());
            let mem    = caps.get(3).map_or("", |m| m.as_str());
            let out = format!(
                "{} | CPU: {} secs | Memory: {} MB |",
                colorize(Color::GoodWord, label),
                colorize(Color::GetTime, cpu),
                colorize(Color::Size, mem),
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
            let cmd  = caps.get(1).map_or("", |m| m.as_str());
            let args = caps.get(2).map_or("", |m| m.as_str());
            let out = format!("{}{}", colorize(Color::Keyword, cmd), colorize(Color::Default, args));
            return PluginResult::Matched(out);
        }

        PluginResult::NoMatch
    }
}
