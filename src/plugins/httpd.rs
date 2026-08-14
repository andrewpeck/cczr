use once_cell::sync::Lazy;
use regex::Regex;

use crate::color::{Color, colorize};
use crate::plugin::{Plugin, PluginResult, PluginType, colorize_http_method, colorize_http_status};

// Combined Apache access log (CLF / vhost-CLF):
//   [vhost ]host ident user [timestamp] "METHOD path HTTP/x" status size [referrer agent]
static RE_ACCESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(\S+)\s(\S+)?\s?-\s(\S+)\s(\[\d{1,2}/\S*/\d{4}:\d{2}:\d{2}:\d{2}[^\]]*\])\s"(([^ "]+)\s*[^"]*)"?\s(\d{3})\s(\d+|-)\s*(.*)$"#,
    )
    .unwrap()
});

// Apache error log:
//   [Day Mon DD HH:MM:SS YYYY] [level] message
static RE_ERROR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(\[\w{3}\s\w{3}\s{1,2}\d{1,2}\s\d{2}:\d{2}:\d{2}\s\d{4}\])\s(\[\w*\])\s(.*)$"#)
        .unwrap()
});

pub struct Httpd;

impl Plugin for Httpd {
    fn name(&self) -> &'static str {
        "httpd"
    }
    fn kind(&self) -> PluginType {
        PluginType::Full
    }

    fn process(&self, line: &str) -> PluginResult {
        if let Some(caps) = RE_ACCESS.captures(line) {
            let host = caps.get(1).map_or("", |m| m.as_str());
            let ident = caps.get(2).map_or("-", |m| m.as_str());
            let user = caps.get(3).map_or("", |m| m.as_str());
            let ts = caps.get(4).map_or("", |m| m.as_str());
            let request = caps.get(5).map_or("", |m| m.as_str());
            let method = caps.get(6).map_or("", |m| m.as_str());
            let status = caps.get(7).map_or("", |m| m.as_str());
            let size = caps.get(8).map_or("", |m| m.as_str());
            let extra = caps.get(9).map_or("", |m| m.as_str());

            // Rebuild request string with colorized method
            let req_colored = if !method.is_empty() {
                let rest = &request[method.len()..];
                format!(
                    "{}{}",
                    colorize_http_method(method),
                    colorize(Color::Uri, rest)
                )
            } else {
                colorize(Color::Uri, request)
            };

            let out = format!(
                r#"{} {} {} {} "{}" {} {} {}"#,
                colorize(Color::Host, host),
                colorize(Color::Ident, ident),
                colorize(Color::User, user),
                colorize(Color::Date, ts),
                req_colored,
                colorize_http_status(status),
                colorize(Color::GetSize, size),
                colorize(Color::Default, extra),
            );
            return PluginResult::Matched(out);
        }

        if let Some(caps) = RE_ERROR.captures(line) {
            let ts = caps.get(1).map_or("", |m| m.as_str());
            let level = caps.get(2).map_or("", |m| m.as_str());
            let msg = caps.get(3).map_or("", |m| m.as_str());

            let level_color = match level.trim_matches(['[', ']']) {
                "error" | "crit" | "alert" | "emerg" => Color::Error,
                "warn" => Color::Warning,
                "notice" | "info" => Color::GoodWord,
                "debug" => Color::Debug,
                _ => Color::Unknown,
            };

            let out = format!(
                "{} {} {}",
                colorize(Color::Date, ts),
                colorize(level_color, level),
                colorize(Color::Default, msg),
            );
            return PluginResult::Matched(out);
        }

        PluginResult::NoMatch
    }
}
