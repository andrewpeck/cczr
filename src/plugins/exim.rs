use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};

// Exim main log timestamp prefix: 2024-01-15 12:34:56
static RE_MAIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s(.*)$").unwrap()
});

// Message with 16-char ID and action symbol
static RE_ACTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S{16})\s([<=*][=>*])\s(\S+.*)$").unwrap()
});

// 16-char ID only
static RE_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S{16})\s(.*)$").unwrap()
});

pub struct Exim;

impl Plugin for Exim {
    fn name(&self) -> &'static str { "exim" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE_MAIN.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };
        let ts   = caps.get(1).map_or("", |m| m.as_str());
        let body = caps.get(2).map_or("", |m| m.as_str());

        let body_colored = if let Some(ac) = RE_ACTION.captures(body) {
            let id     = ac.get(1).map_or("", |m| m.as_str());
            let action = ac.get(2).map_or("", |m| m.as_str());
            let rest   = ac.get(3).map_or("", |m| m.as_str());
            let a_color = match action {
                "<=" => Color::Incoming,
                "=>" => Color::Outgoing,
                _    => Color::Default,
            };
            format!(
                "{} {} {}",
                colorize(Color::UniqueId, id),
                colorize(a_color, action),
                colorize(Color::Default, rest),
            )
        } else if let Some(ic) = RE_ID.captures(body) {
            let id   = ic.get(1).map_or("", |m| m.as_str());
            let rest = ic.get(2).map_or("", |m| m.as_str());
            format!("{} {}", colorize(Color::UniqueId, id), colorize(Color::Default, rest))
        } else {
            colorize(Color::Default, body)
        };

        let out = format!("{} {}", colorize(Color::Date, ts), body_colored);
        PluginResult::Matched(out)
    }
}
