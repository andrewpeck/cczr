use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{Color, colorize};
use crate::plugin::{Plugin, PluginResult, PluginType};

// Syslog prefix (reused from syslog plugin pattern)
static RE_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S+\s{1,2}\d{1,2}\s\d{2}:\d{2}:\d{2})\s(\S+)\s+postfix/(\S+)\[(\d+)\]:\s(.*)$")
        .unwrap()
});

// Queue-id: field=value pairs
static RE_QUEUE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([0-9A-F]+): (.*)$").unwrap());

static RE_KV: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\w-]+)=([^,\s]+)").unwrap());

pub struct Postfix;

impl Plugin for Postfix {
    fn name(&self) -> &'static str {
        "postfix"
    }
    fn kind(&self) -> PluginType {
        PluginType::Full
    }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE_PREFIX.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };

        let date = caps.get(1).map_or("", |m| m.as_str());
        let host = caps.get(2).map_or("", |m| m.as_str());
        let service = caps.get(3).map_or("", |m| m.as_str());
        let pid = caps.get(4).map_or("", |m| m.as_str());
        let body = caps.get(5).map_or("", |m| m.as_str());

        let body_colored = if let Some(qc) = RE_QUEUE.captures(body) {
            let qid = qc.get(1).map_or("", |m| m.as_str());
            let rest = qc.get(2).map_or("", |m| m.as_str());
            // colorize key=value pairs
            let rest_colored = RE_KV.replace_all(rest, |c: &regex::Captures<'_>| {
                let key = c.get(1).map_or("", |m| m.as_str());
                let val = c.get(2).map_or("", |m| m.as_str());
                let val_color = match key {
                    "from" | "to" => Color::Email,
                    "status" => Color::HttpCodes,
                    "host" => Color::Host,
                    "size" => Color::Size,
                    _ => Color::Default,
                };
                format!(
                    "{}={}",
                    colorize(Color::Field, key),
                    colorize(val_color, val)
                )
            });
            format!("{}: {}", colorize(Color::UniqueId, qid), rest_colored)
        } else {
            colorize(Color::Default, body)
        };

        let out = format!(
            "{} {} postfix/{}[{}]: {}",
            colorize(Color::Date, date),
            colorize(Color::Host, host),
            colorize(Color::Service, service),
            colorize(Color::Pid, pid),
            body_colored,
        );
        PluginResult::Matched(out)
    }
}
