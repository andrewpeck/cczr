use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};

// SU MM/DD HH:MM +/- tty from_user-to_user
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^SU (\d{2}/\d{2} \d{2}:\d{2}) ([+\-]) (\S+) ([^\-]+)-(.*)$").unwrap()
});

pub struct Sulog;

impl Plugin for Sulog {
    fn name(&self) -> &'static str { "sulog" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        let caps = match RE.captures(line) {
            Some(c) => c,
            None => return PluginResult::NoMatch,
        };
        let ts      = caps.get(1).map_or("", |m| m.as_str());
        let status  = caps.get(2).map_or("", |m| m.as_str());
        let tty     = caps.get(3).map_or("", |m| m.as_str());
        let from    = caps.get(4).map_or("", |m| m.as_str()).trim();
        let to      = caps.get(5).map_or("", |m| m.as_str()).trim();

        let s_color = if status == "+" { Color::GoodWord } else { Color::BadWord };

        let out = format!(
            "SU {} {} {} {}-{}",
            colorize(Color::Date, ts),
            colorize(s_color, status),
            colorize(Color::Service, tty),
            colorize(Color::User, from),
            colorize(Color::User, to),
        );
        PluginResult::Matched(out)
    }
}
