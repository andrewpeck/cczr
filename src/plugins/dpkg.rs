use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult, PluginType};

// 2024-01-15 12:34:56 status <state> <pkg> <version>
static RE_STATUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([-\d]{10}\s[:\d]{8})\sstatus\s(\S+)\s(\S+)\s(\S+)$").unwrap()
});

// 2024-01-15 12:34:56 install|upgrade|remove|purge <pkg> <old> <new>
static RE_ACTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([-\d]{10}\s[:\d]{8})\s(install|upgrade|remove|purge)\s(\S+)\s(\S+)\s(\S+)$")
        .unwrap()
});

// 2024-01-15 12:34:56 conffile <file> install|keep
static RE_CONFFILE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([-\d]{10}\s[:\d]{8})\sconffile\s(\S+)\s(install|keep)$").unwrap()
});

pub struct Dpkg;

impl Plugin for Dpkg {
    fn name(&self) -> &'static str { "dpkg" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        if let Some(caps) = RE_STATUS.captures(line) {
            let ts    = caps.get(1).map_or("", |m| m.as_str());
            let state = caps.get(2).map_or("", |m| m.as_str());
            let pkg   = caps.get(3).map_or("", |m| m.as_str());
            let ver   = caps.get(4).map_or("", |m| m.as_str());
            let out = format!(
                "{} status {} {} {}",
                colorize(Color::Date, ts),
                colorize(Color::PkgStatus, state),
                colorize(Color::Package, pkg),
                colorize(Color::Version, ver),
            );
            return PluginResult::Matched(out);
        }

        if let Some(caps) = RE_ACTION.captures(line) {
            let ts     = caps.get(1).map_or("", |m| m.as_str());
            let action = caps.get(2).map_or("", |m| m.as_str());
            let pkg    = caps.get(3).map_or("", |m| m.as_str());
            let old    = caps.get(4).map_or("", |m| m.as_str());
            let new    = caps.get(5).map_or("", |m| m.as_str());
            let a_color = match action {
                "install" | "upgrade" => Color::GoodWord,
                "remove"  | "purge"   => Color::BadWord,
                _                     => Color::Default,
            };
            let out = format!(
                "{} {} {} {} {}",
                colorize(Color::Date, ts),
                colorize(a_color, action),
                colorize(Color::Package, pkg),
                colorize(Color::Version, old),
                colorize(Color::Version, new),
            );
            return PluginResult::Matched(out);
        }

        if let Some(caps) = RE_CONFFILE.captures(line) {
            let ts      = caps.get(1).map_or("", |m| m.as_str());
            let path    = caps.get(2).map_or("", |m| m.as_str());
            let decision= caps.get(3).map_or("", |m| m.as_str());
            let out = format!(
                "{} conffile {} {}",
                colorize(Color::Date, ts),
                colorize(Color::File, path),
                colorize(Color::Keyword, decision),
            );
            return PluginResult::Matched(out);
        }

        PluginResult::NoMatch
    }
}
