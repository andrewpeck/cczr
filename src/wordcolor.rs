use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};

static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}").unwrap()
});
static RE_IP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
});
static RE_MAC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:[0-9a-f]{2}:){5}[0-9a-f]{2}\b").unwrap()
});
static RE_URI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\w{2,}://\S+").unwrap()
});
static RE_DIR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|\s)((?:/[^\s/]+)+/?)").unwrap()
});
static RE_SIZE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b\d+(?:\.\d+)?[KMGT]?B\b").unwrap()
});
static RE_VERSION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\bv?\d+\.\d+(?:\.\d+)*\b").unwrap()
});
static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{2}:\d{2}:\d{2}\b").unwrap()
});
static RE_ADDRESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b0x[0-9a-fA-F]+\b").unwrap()
});
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d+\b").unwrap()
});

static BAD_WORDS: &[&str] = &[
    "warn", "error", "fail", "unable", "offline", "ignored", "bad",
    "critical", "severe", "denied", "refused", "unavailable",
    "disconnected", "unreachable", "invalid", "timeout", "dropped",
    "lost", "shutdown", "terminated", "disabled", "forbidden",
    "uninitialized", "failed", "broken", "unsupported",
];

static GOOD_WORDS: &[&str] = &[
    "start", "online", "ready", "complete", "detect", "success",
    "loaded", "found", "open", "connected", "enabled", "running",
    "operational", "initialized", "launched", "opened", "up",
    "listening", "active", "activated",
];

static SYSTEM_WORDS: &[&str] = &[
    "kernel", "cpu", "bios", "firmware", "driver", "device",
    "interface", "module", "subsystem", "layer", "stack", "bus",
    "controller", "usb", "pci", "scsi", "raid", "ata",
];

fn word_color(word: &str) -> Color {
    let lower = word.to_lowercase();
    if BAD_WORDS.iter().any(|&w| lower.contains(w)) {
        return Color::BadWord;
    }
    if GOOD_WORDS.iter().any(|&w| lower == w) {
        return Color::GoodWord;
    }
    if word == "INFO" {
        return Color::Info;
    }
    if SYSTEM_WORDS.iter().any(|&w| lower.contains(w)) {
        return Color::SystemWord;
    }
    Color::Default
}

/// Apply word-level colorization to a plain-text string that was not matched
/// by any plugin. Returns the colorized string.
pub fn colorize_words(input: &str) -> String {
    // We run a series of non-overlapping replacements on the raw text.
    // Strategy: build an interval list of (start, end, color, text) from
    // all regex matches, then reconstruct the string.

    let mut spans: Vec<(usize, usize, Color)> = Vec::new();

    let add_spans = |re: &Regex, color: Color, text: &str, spans: &mut Vec<(usize, usize, Color)>| {
        for m in re.find_iter(text) {
            spans.push((m.start(), m.end(), color));
        }
    };

    add_spans(&RE_EMAIL,   Color::Email,   input, &mut spans);
    add_spans(&RE_MAC,     Color::Mac,     input, &mut spans);
    add_spans(&RE_URI,     Color::Uri,     input, &mut spans);
    add_spans(&RE_IP,      Color::Host,    input, &mut spans);
    add_spans(&RE_ADDRESS, Color::Address, input, &mut spans);
    add_spans(&RE_SIZE,    Color::Size,    input, &mut spans);
    add_spans(&RE_VERSION, Color::Version, input, &mut spans);
    add_spans(&RE_TIME,    Color::Date,    input, &mut spans);
    add_spans(&RE_NUMBER,  Color::Numbers, input, &mut spans);

    // For directory paths, use the capture group
    for cap in RE_DIR.captures_iter(input) {
        if let Some(m) = cap.get(1) {
            spans.push((m.start(), m.end(), Color::Dir));
        }
    }

    // Remove overlapping spans: sort by start, then greedily pick non-overlapping
    spans.sort_by_key(|&(s, e, _)| (s, std::cmp::Reverse(e)));
    let mut filtered: Vec<(usize, usize, Color)> = Vec::new();
    let mut cursor = 0usize;
    for (s, e, c) in spans {
        if s >= cursor {
            filtered.push((s, e, c));
            cursor = e;
        }
    }

    // Reconstruct the string
    let mut out = String::with_capacity(input.len() * 2);
    let bytes = input.as_bytes();
    let mut pos = 0usize;

    for (s, e, color) in &filtered {
        if pos < *s {
            // Colorize plain words between spans
            let between = &input[pos..*s];
            out.push_str(&colorize_plain_words(between));
        }
        out.push_str(&colorize(color.clone(), &input[*s..*e]));
        pos = *e;
    }
    if pos < bytes.len() {
        out.push_str(&colorize_plain_words(&input[pos..]));
    }
    out
}

fn colorize_plain_words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split_inclusive(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let trimmed = word.trim_matches(|c: char| {
            c.is_whitespace() || matches!(c, ',' | ';' | ':' | '.' | '!' | '?' | '[' | ']' | '(' | ')')
        });
        if trimmed.is_empty() {
            out.push_str(word);
            continue;
        }
        let color = word_color(trimmed);
        if color != Color::Default {
            // Reconstruct: prefix + colored word + suffix
            let start = word.find(trimmed).unwrap_or(0);
            let end = start + trimmed.len();
            out.push_str(&word[..start]);
            out.push_str(&colorize(color, trimmed));
            out.push_str(&word[end..]);
        } else {
            out.push_str(word);
        }
    }
    out
}

