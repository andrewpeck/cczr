pub mod color;
pub mod plugin;
pub mod plugins;
pub mod wordcolor;

#[cfg(test)]
mod tests;

use plugin::{Plugin, PluginResult, PluginType};

/// Build the default ordered list of plugins.
/// Full plugins are tried first in order; if none match, partial plugins run.
pub fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(plugins::Vivado),
        Box::new(plugins::Syslog),
        Box::new(plugins::Httpd),
        Box::new(plugins::Postfix),
        Box::new(plugins::Squid),
        Box::new(plugins::Exim),
        Box::new(plugins::Dpkg),
        Box::new(plugins::Php),
        Box::new(plugins::Vsftpd),
        Box::new(plugins::Sulog),
        // Partial plugins last
        Box::new(plugins::Ulogd),
    ]
}

/// Colorize a single log line.
/// Tries full plugins first, then partial, then falls back to word colorization.
pub fn colorize_line(line: &str, plugins: &[Box<dyn Plugin>]) -> String {
    // Try FULL plugins
    for p in plugins.iter().filter(|p| p.kind() == PluginType::Full) {
        if let PluginResult::Matched(out) = p.process(line) {
            return out;
        }
    }

    // Try PARTIAL plugins — apply all that match, chaining output
    let mut current = line.to_owned();
    let mut any_partial = false;
    for p in plugins.iter().filter(|p| p.kind() == PluginType::Partial) {
        if let PluginResult::Matched(out) = p.process(&current) {
            current = out;
            any_partial = true;
        }
    }
    if any_partial {
        return current;
    }

    // Fallback: word-level colorization
    wordcolor::colorize_words(line)
}
