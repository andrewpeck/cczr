use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};

// [DD-Mon-YYYY HH:MM:SS] PHP <message>
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\[\d+-\w+-\d+ \d+:\d+:\d+\]) PHP (.*)$").unwrap()
});

pub struct Php;

impl Plugin for Php {
    fn name(&self) -> &'static str { "php" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };
        let ts  = caps.get(1).map_or("", |m| m.as_str());
        let msg = caps.get(2).map_or("", |m| m.as_str());

        // Detect severity from message prefix
        let msg_color = if msg.starts_with("Fatal") || msg.starts_with("Parse error") {
            Color::Error
        } else if msg.starts_with("Warning") || msg.starts_with("Notice") {
            Color::Warning
        } else {
            Color::Default
        };

        let out = format!(
            "{} PHP {}",
            colorize(Color::Date, ts),
            colorize(msg_color, msg),
        );
        PluginResult::Matched(out)
    }
}
