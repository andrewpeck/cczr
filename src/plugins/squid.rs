use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{colorize, Color};
use crate::plugin::{colorize_http_status, Plugin, PluginResult, PluginType};

// Squid access.log (native format)
static RE_ACCESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(\d{9,10}\.\d{3})(\s+)(\d+)\s(\S+)\s(\w+)/(\d{3})\s(\d+)\s(\w+)\s(\S+)\s(\S+)\s(\w+)/([\d.]+|-)\s(.*)"
    ).unwrap()
});

// Squid store.log
static RE_STORE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^([\d.]+)\s(\w+)\s(-?[0-9A-F]+)\s+(\S+)\s([0-9A-F]+)(\s+)(\d{3}|\?)(\s+)(-?[\d?]+)(\s+)(-?[\d?]+)(\s+)(-?[\d?]+)\s(\S+)\s(-?[\d|?]+)/(-?[\d|?]+)\s(\S+)\s(.*)"
    ).unwrap()
});

// Squid cache.log
static RE_CACHE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{4}/\d{2}/\d{2}\s(?:\d{2}:){2}\d{2}\|)\s(.*)$").unwrap()
});

fn colorize_squid_result(result: &str) -> String {
    let color = if result.contains("HIT") {
        Color::ProxyHit
    } else if result.contains("MISS") {
        Color::ProxyMiss
    } else if result.contains("DENIED") {
        Color::ProxyDenied
    } else if result.contains("REFRESH") {
        Color::ProxyRefresh
    } else {
        Color::Unknown
    };
    colorize(color, result)
}

pub struct Squid;

impl Plugin for Squid {
    fn name(&self) -> &'static str { "squid" }
    fn kind(&self) -> PluginType { PluginType::Full }

    fn process(&self, line: &str) -> PluginResult {
        if let Some(caps) = RE_ACCESS.captures(line) {
            let ts      = caps.get(1).map_or("", |m| m.as_str());
            let elapsed = caps.get(3).map_or("", |m| m.as_str());
            let client  = caps.get(4).map_or("", |m| m.as_str());
            let result  = caps.get(5).map_or("", |m| m.as_str());
            let status  = caps.get(6).map_or("", |m| m.as_str());
            let size    = caps.get(7).map_or("", |m| m.as_str());
            let method  = caps.get(8).map_or("", |m| m.as_str());
            let url     = caps.get(9).map_or("", |m| m.as_str());
            let ident   = caps.get(10).map_or("", |m| m.as_str());
            let hier    = caps.get(11).map_or("", |m| m.as_str());
            let peer    = caps.get(12).map_or("", |m| m.as_str());
            let ctype   = caps.get(13).map_or("", |m| m.as_str());

            let out = format!(
                "{} {} {} {}/{} {} {} {} {} {}/{}  {}",
                colorize(Color::Date, ts),
                colorize(Color::GetTime, elapsed),
                colorize(Color::Host, client),
                colorize_squid_result(result),
                colorize_http_status(status),
                colorize(Color::GetSize, size),
                colorize(Color::HttpGet, method),
                colorize(Color::Uri, url),
                colorize(Color::Ident, ident),
                colorize(Color::ProxyDirect, hier),
                colorize(Color::Host, peer),
                colorize(Color::ContentType, ctype),
            );
            return PluginResult::Matched(out);
        }

        if let Some(caps) = RE_STORE.captures(line) {
            let ts     = caps.get(1).map_or("", |m| m.as_str());
            let action = caps.get(2).map_or("", |m| m.as_str());
            let key    = caps.get(4).map_or("", |m| m.as_str());

            let action_color = match action {
                "CREATE"  => Color::ProxyCreate,
                "RELEASE" => Color::ProxyRelease,
                "SWAPIN"  => Color::ProxySwapIn,
                "SWAPOUT" => Color::ProxySwapOut,
                _         => Color::Unknown,
            };
            let out = format!(
                "{} {} {} …",
                colorize(Color::Date, ts),
                colorize(action_color, action),
                colorize(Color::Uri, key),
            );
            return PluginResult::Matched(out);
        }

        if let Some(caps) = RE_CACHE.captures(line) {
            let ts  = caps.get(1).map_or("", |m| m.as_str());
            let msg = caps.get(2).map_or("", |m| m.as_str());
            let out = format!("{} {}", colorize(Color::Date, ts), colorize(Color::Default, msg));
            return PluginResult::Matched(out);
        }

        PluginResult::NoMatch
    }
}
