use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};

// Timestamp  [pid N] [(user)] message
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\S+\s+\S+\s+\d{1,2}\s+\d{1,2}:\d{1,2}:\d{1,2}\s+\d+)(\s+)\[pid (\d+)\]\s(\[(\S+)\])?\s*(.*)$",
    )
    .unwrap()
});

pub struct Vsftpd;

impl Plugin for Vsftpd {
    fn name(&self) -> &'static str { "vsftpd" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };
        let ts   = caps.get(1).map_or("", |m| m.as_str());
        let pid  = caps.get(3).map_or("", |m| m.as_str());
        let user = caps.get(5).map_or("", |m| m.as_str());
        let msg  = caps.get(6).map_or("", |m| m.as_str());

        let user_part = if user.is_empty() {
            String::new()
        } else {
            format!("[{}] ", colorize(Color::User, user))
        };

        let out = format!(
            "{} [pid {}] {}{}",
            colorize(Color::Date, ts),
            colorize(Color::Pid, pid),
            user_part,
            colorize(Color::Default, msg),
        );
        PluginResult::Matched(out)
    }
}
