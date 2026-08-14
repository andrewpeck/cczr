use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{Color, colorize};
use crate::plugin::{Plugin, PluginResult, PluginType};
use crate::wordcolor::colorize_words;

// Month Day HH:MM:SS  hostname  process[pid]: message
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S+\s{1,2}\d{1,2}\s\d{2}:\d{2}:\d{2})\s(\S+)\s+((\S+?):?\s(.*))$").unwrap()
});

// process[pid] or process(pid)
static RE_PROC: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^\[\(]+)[\[\(](\d+)[\]\)]$").unwrap());

pub struct Syslog;

impl Plugin for Syslog {
    fn name(&self) -> &'static str {
        "syslog"
    }
    fn kind(&self) -> PluginType {
        PluginType::Full
    }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };

        let date = caps.get(1).map_or("", |m| m.as_str());
        let host = caps.get(2).map_or("", |m| m.as_str());
        let proc_full = caps.get(4).map_or("", |m| m.as_str());
        let msg = caps.get(5).map_or("", |m| m.as_str());

        let proc_colored = if let Some(pc) = RE_PROC.captures(proc_full) {
            let pname = pc.get(1).map_or("", |m| m.as_str());
            let pid = pc.get(2).map_or("", |m| m.as_str());
            format!(
                "{}[{}]",
                colorize(Color::Process, pname),
                colorize(Color::Pid, pid)
            )
        } else {
            colorize(Color::Process, proc_full)
        };

        let out = format!(
            "{} {} {}: {}",
            colorize(Color::Date, date),
            colorize(Color::Host, host),
            proc_colored,
            colorize_words(msg),
        );
        PluginResult::Matched(out)
    }
}
