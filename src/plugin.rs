use crate::color::{colorize, Color};

/// Whether a plugin handles the full line or only parts of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Plugin consumes the entire line.
    Full,
    /// Plugin only colorizes certain sub-strings; rest passed on.
    Partial,
}

/// Result from a plugin attempt.
pub enum PluginResult {
    /// Plugin matched and produced colorized output.
    Matched(String),
    /// Plugin did not match this line.
    NoMatch,
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> PluginType;
    /// Try to colorize `line`.  Returns `Matched(colorized)` or `NoMatch`.
    fn process(&self, line: &str) -> PluginResult;
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Colorize an HTTP status code string.
pub(crate) fn colorize_http_status(code: &str) -> String {
    let color = match code.chars().next() {
        Some('2') => Color::HttpCodes,
        Some('3') => Color::HttpCodes,
        Some('4') => Color::Error,
        Some('5') => Color::Error,
        _ => Color::Unknown,
    };
    colorize(color, code)
}

/// Colorize an HTTP method string.
pub(crate) fn colorize_http_method(method: &str) -> String {
    let color = match method {
        "GET"     => Color::HttpGet,
        "POST"    => Color::HttpPost,
        "HEAD"    => Color::HttpHead,
        "PUT"     => Color::HttpPut,
        "CONNECT" => Color::HttpConnect,
        "TRACE"   => Color::HttpTrace,
        _         => Color::Unknown,
    };
    colorize(color, method)
}
