use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{Color, colorize};
use crate::plugin::{Plugin, PluginResult, PluginType};

// Colorize netfilter field names: IN=, OUT=, SRC=, DST=, etc.
static RE_FIELD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(IN|OUT|MAC|TTL|SRC|DST|TOS|PREC|SPT|DPT|PROTO|LEN|ID|DF|WINDOW|RES|SYN|URGP)=(\S*)",
    )
    .unwrap()
});

pub struct Ulogd;

impl Plugin for Ulogd {
    fn name(&self) -> &'static str {
        "ulogd"
    }
    fn kind(&self) -> PluginType {
        PluginType::Partial
    }

    fn process(&self, line: &str) -> PluginResult {
        if !RE_FIELD.is_match(line) {
            return PluginResult::NoMatch;
        }
        let result = RE_FIELD.replace_all(line, |caps: &regex::Captures<'_>| {
            let key = caps.get(1).map_or("", |m| m.as_str());
            let val = caps.get(2).map_or("", |m| m.as_str());
            let val_color = match key {
                "SRC" | "DST" => Color::Host,
                "MAC" => Color::Mac,
                "PROTO" => Color::Protocol,
                "IN" | "OUT" => Color::Service,
                _ => Color::Numbers,
            };
            format!(
                "{}={}",
                colorize(Color::Field, key),
                colorize(val_color, val)
            )
        });
        PluginResult::Matched(result.into_owned())
    }
}
