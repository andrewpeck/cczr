/// Integration-style tests for each plugin.
/// Each test verifies that:
///   1. The plugin matches its intended log format.
///   2. The output contains the expected ANSI-coloured substrings.
///   3. The plugin does NOT match lines that belong to a different format.
///
/// Log samples are drawn from or closely modelled on the CCZE test corpus:
///   https://github.com/cornet/ccze
use crate::color::{colorize, Color};
use crate::plugin::{Plugin, PluginResult};
use crate::plugins::*;
use crate::{colorize_line, default_plugins};

// ─── helpers ────────────────────────────────────────────────────────────────

fn matched(result: PluginResult) -> String {
    match result {
        PluginResult::Matched(s) => s,
        PluginResult::NoMatch => panic!("expected Matched, got NoMatch"),
    }
}

fn assert_no_match(result: PluginResult) {
    match result {
        PluginResult::NoMatch => {}
        PluginResult::Matched(_) => panic!("expected NoMatch but got Matched"),
    }
}

fn contains_colored(output: &str, color: Color, text: &str) -> bool {
    output.contains(&colorize(color, text))
}

// ─── Syslog ─────────────────────────────────────────────────────────────────

#[test]
fn syslog_basic() {
    let line = "Jan  5 12:34:56 myhost sshd[1234]: Accepted password for alice";
    let out = matched(Syslog.process(line));
    assert!(contains_colored(&out, Color::Date, "Jan  5 12:34:56"), "date");
    assert!(contains_colored(&out, Color::Host, "myhost"), "host");
    assert!(contains_colored(&out, Color::Process, "sshd"), "process");
    assert!(contains_colored(&out, Color::Pid, "1234"), "pid");
    assert!(out.contains("Accepted password for alice"), "message");
}

#[test]
fn syslog_no_pid() {
    let line = "Mar 15 08:00:01 router dhcpd: DHCPREQUEST from 192.168.1.10";
    let out = matched(Syslog.process(line));
    assert!(contains_colored(&out, Color::Date, "Mar 15 08:00:01"), "date");
    assert!(contains_colored(&out, Color::Host, "router"), "host");
    assert!(contains_colored(&out, Color::Process, "dhcpd"), "process");
}

#[test]
fn syslog_double_space_day() {
    let line = "Jan  1 00:00:00 host kernel: Oops: general protection fault";
    let out = matched(Syslog.process(line));
    assert!(contains_colored(&out, Color::Date, "Jan  1 00:00:00"));
}

#[test]
fn syslog_no_match_on_apache_access() {
    let line = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326"#;
    assert_no_match(Syslog.process(line));
}

// ─── Apache httpd ────────────────────────────────────────────────────────────

#[test]
fn httpd_access_get() {
    let line = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326"#;
    let out = matched(Httpd.process(line));
    assert!(contains_colored(&out, Color::Host, "127.0.0.1"), "host");
    assert!(contains_colored(&out, Color::User, "frank"), "user");
    assert!(contains_colored(&out, Color::Date, "[10/Oct/2000:13:55:36 -0700]"), "date");
    assert!(contains_colored(&out, Color::HttpGet, "GET"), "method");
    assert!(contains_colored(&out, Color::HttpCodes, "200"), "status 2xx");
    assert!(contains_colored(&out, Color::GetSize, "2326"), "size");
}

#[test]
fn httpd_access_404() {
    let line = r#"10.0.0.1 - - [01/Jan/2024:00:00:00 +0000] "GET /missing HTTP/1.1" 404 512"#;
    let out = matched(Httpd.process(line));
    assert!(contains_colored(&out, Color::Error, "404"), "404 → Error color");
}

#[test]
fn httpd_access_post() {
    let line = r#"192.168.0.5 - admin [15/Mar/2024:10:20:30 +0100] "POST /api/data HTTP/1.1" 201 128"#;
    let out = matched(Httpd.process(line));
    assert!(contains_colored(&out, Color::HttpPost, "POST"), "POST method color");
}

#[test]
fn httpd_error_log() {
    let line = "[Wed Jan 15 12:00:00 2025] [error] File not found: /var/www/missing.html";
    let out = matched(Httpd.process(line));
    assert!(contains_colored(&out, Color::Date, "[Wed Jan 15 12:00:00 2025]"), "error log date");
    assert!(contains_colored(&out, Color::Error, "[error]"), "error level");
    assert!(out.contains("File not found"), "message preserved");
}

#[test]
fn httpd_no_match_on_syslog() {
    let line = "Jan  5 12:34:56 myhost sshd[1234]: Accepted password for alice";
    assert_no_match(Httpd.process(line));
}

// ─── Postfix ─────────────────────────────────────────────────────────────────

#[test]
fn postfix_basic() {
    let line = "Jan 15 09:00:01 mail postfix/smtp[456]: 3A1B2C3D4E: to=<user@example.com>, relay=mx.example.com[1.2.3.4]:25, status=sent";
    let out = matched(Postfix.process(line));
    assert!(contains_colored(&out, Color::Date, "Jan 15 09:00:01"), "date");
    assert!(contains_colored(&out, Color::Host, "mail"), "host");
    assert!(contains_colored(&out, Color::Service, "smtp"), "service");
    assert!(contains_colored(&out, Color::Pid, "456"), "pid");
    assert!(contains_colored(&out, Color::UniqueId, "3A1B2C3D4E"), "queue id");
}

#[test]
fn postfix_no_match() {
    assert_no_match(Postfix.process("Jan  5 12:34:56 myhost sshd[1234]: message"));
}

// ─── Squid ───────────────────────────────────────────────────────────────────

#[test]
fn squid_access_hit() {
    let line = "1234567890.123   500 192.168.1.1 TCP_HIT/200 4096 GET http://example.com/ alice NONE/- text/html";
    let out = matched(Squid.process(line));
    assert!(contains_colored(&out, Color::Date, "1234567890.123"), "ts");
    assert!(contains_colored(&out, Color::Host, "192.168.1.1"), "client");
    assert!(contains_colored(&out, Color::ProxyHit, "TCP_HIT"), "cache result");
    assert!(contains_colored(&out, Color::HttpCodes, "200"), "status");
    assert!(contains_colored(&out, Color::HttpGet, "GET"), "method");
}

#[test]
fn squid_access_miss() {
    let line = "1234567890.000   100 10.0.0.2 TCP_MISS/404 512 GET http://missing.test/ - NONE/- -";
    let out = matched(Squid.process(line));
    assert!(contains_colored(&out, Color::ProxyMiss, "TCP_MISS"), "cache miss");
    assert!(contains_colored(&out, Color::Error, "404"), "404 status");
}

#[test]
fn squid_cache_log() {
    let line = "2024/01/15 12:00:00| Starting Squid Cache version 5.0";
    let out = matched(Squid.process(line));
    assert!(contains_colored(&out, Color::Date, "2024/01/15 12:00:00|"), "cache date");
}

#[test]
fn squid_no_match() {
    assert_no_match(Squid.process("Jan  5 12:34:56 host proc[1]: msg"));
}

// ─── Exim ────────────────────────────────────────────────────────────────────

#[test]
fn exim_delivery() {
    let line = "2024-01-15 12:00:00 1rABCD-000001-00 => user@example.com R=dnslookup T=remote_smtp";
    let out = matched(Exim.process(line));
    assert!(contains_colored(&out, Color::Date, "2024-01-15 12:00:00"), "date");
    assert!(contains_colored(&out, Color::UniqueId, "1rABCD-000001-00"), "id");
    assert!(contains_colored(&out, Color::Outgoing, "=>"), "outgoing");
}

#[test]
fn exim_received() {
    let line = "2024-01-15 12:00:00 1rABCD-000001-00 <= sender@example.com H=mx.example.com";
    let out = matched(Exim.process(line));
    assert!(contains_colored(&out, Color::Incoming, "<="), "incoming");
}

#[test]
fn exim_no_match() {
    assert_no_match(Exim.process("Jan  5 12:34:56 host exim[1]: message"));
}

// ─── Dpkg ────────────────────────────────────────────────────────────────────

#[test]
fn dpkg_status() {
    let line = "2024-01-15 12:00:00 status installed curl 7.88.1-1";
    let out = matched(Dpkg.process(line));
    assert!(contains_colored(&out, Color::Date, "2024-01-15 12:00:00"), "date");
    assert!(contains_colored(&out, Color::PkgStatus, "installed"), "status");
    assert!(contains_colored(&out, Color::Package, "curl"), "package");
    assert!(contains_colored(&out, Color::Version, "7.88.1-1"), "version");
}

#[test]
fn dpkg_install() {
    let line = "2024-01-15 12:00:00 install curl <none> 7.88.1-1";
    let out = matched(Dpkg.process(line));
    assert!(contains_colored(&out, Color::GoodWord, "install"), "install → GoodWord");
    assert!(contains_colored(&out, Color::Package, "curl"), "package");
}

#[test]
fn dpkg_remove() {
    let line = "2024-01-15 12:00:00 remove curl 7.88.1-1 <none>";
    let out = matched(Dpkg.process(line));
    assert!(contains_colored(&out, Color::BadWord, "remove"), "remove → BadWord");
}

#[test]
fn dpkg_conffile() {
    let line = "2024-01-15 12:00:00 conffile /etc/curl/curlrc install";
    let out = matched(Dpkg.process(line));
    assert!(contains_colored(&out, Color::File, "/etc/curl/curlrc"), "conffile path");
    assert!(contains_colored(&out, Color::Keyword, "install"), "decision");
}

#[test]
fn dpkg_no_match() {
    assert_no_match(Dpkg.process("Jan  5 12:34:56 host dpkg[1]: message"));
}

// ─── PHP ─────────────────────────────────────────────────────────────────────

#[test]
fn php_fatal() {
    let line = "[15-Jan-2024 12:00:00] PHP Fatal error: Call to undefined function foo()";
    let out = matched(Php.process(line));
    assert!(contains_colored(&out, Color::Date, "[15-Jan-2024 12:00:00]"), "date");
    assert!(contains_colored(&out, Color::Error, "Fatal error: Call to undefined function foo()"), "fatal msg");
}

#[test]
fn php_warning() {
    let line = "[15-Jan-2024 12:00:00] PHP Warning: Division by zero";
    let out = matched(Php.process(line));
    assert!(contains_colored(&out, Color::Warning, "Warning: Division by zero"), "warning msg");
}

#[test]
fn php_no_match() {
    assert_no_match(Php.process("Jan  5 12:34:56 host php[1]: message"));
}

// ─── Vsftpd ──────────────────────────────────────────────────────────────────

#[test]
fn vsftpd_basic() {
    let line = "Mon Jan 15 12:00:00 2024 [pid 789] [alice] CONNECT: Client 192.168.1.1";
    let out = matched(Vsftpd.process(line));
    assert!(contains_colored(&out, Color::Date, "Mon Jan 15 12:00:00 2024"), "date");
    assert!(contains_colored(&out, Color::Pid, "789"), "pid");
    assert!(contains_colored(&out, Color::User, "alice"), "user");
    assert!(out.contains("CONNECT"), "message");
}

#[test]
fn vsftpd_no_user() {
    let line = "Mon Jan 15 12:00:00 2024 [pid 790] CONNECT: Client 10.0.0.1";
    let out = matched(Vsftpd.process(line));
    assert!(contains_colored(&out, Color::Pid, "790"), "pid");
    assert!(!out.contains("[color:User]"), "no user color");
}

#[test]
fn vsftpd_no_match() {
    assert_no_match(Vsftpd.process("Jan  5 12:34:56 host vsftpd: message"));
}

// ─── Sulog ───────────────────────────────────────────────────────────────────

#[test]
fn sulog_success() {
    let line = "SU 01/15 12:00 + pts/0 alice-root";
    let out = matched(Sulog.process(line));
    assert!(contains_colored(&out, Color::Date, "01/15 12:00"), "date");
    assert!(contains_colored(&out, Color::GoodWord, "+"), "success");
    assert!(contains_colored(&out, Color::User, "alice"), "from user");
    assert!(contains_colored(&out, Color::User, "root"), "to user");
}

#[test]
fn sulog_failure() {
    let line = "SU 01/15 12:01 - pts/1 bob-root";
    let out = matched(Sulog.process(line));
    assert!(contains_colored(&out, Color::BadWord, "-"), "failure color");
}

#[test]
fn sulog_no_match() {
    assert_no_match(Sulog.process("Jan  5 12:34:56 host kernel: message"));
}

// ─── Ulogd (partial) ─────────────────────────────────────────────────────────

#[test]
fn ulogd_netfilter() {
    let line = "IN=eth0 OUT= MAC=00:11:22:33:44:55 SRC=1.2.3.4 DST=5.6.7.8 PROTO=TCP SPT=1234 DPT=80";
    let out = matched(Ulogd.process(line));
    assert!(contains_colored(&out, Color::Field, "SRC"), "SRC field");
    assert!(contains_colored(&out, Color::Host, "1.2.3.4"), "src ip");
    assert!(contains_colored(&out, Color::Host, "5.6.7.8"), "dst ip");
    assert!(contains_colored(&out, Color::Mac, "00:11:22:33:44:55"), "mac");
    assert!(contains_colored(&out, Color::Protocol, "TCP"), "protocol");
    assert!(contains_colored(&out, Color::Service, "eth0"), "interface");
}

#[test]
fn ulogd_no_match() {
    assert_no_match(Ulogd.process("no netfilter fields here at all"));
}

// ─── Vivado ───────────────────────────────────────────────────────────────────

#[test]
fn vivado_info() {
    let line = "INFO: [Netlist 29-17] Analyzing 638 Unisim elements for replacement";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Keyword, "INFO"), "INFO level");
    assert!(contains_colored(&out, Color::Ident, "[Netlist 29-17]"), "tag");
    assert!(out.contains("Analyzing"), "message body");
}

#[test]
fn vivado_warning() {
    let line = "WARNING: [Board 49-26] cannot add Board Part xilinx.com:vek280:part0:1.0";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Warning, "WARNING"), "WARNING level");
    assert!(contains_colored(&out, Color::Ident, "[Board 49-26]"), "tag");
}

#[test]
fn vivado_critical_warning() {
    let line = "CRITICAL WARNING: [Timing 38-282] The design failed to meet the timing requirements.";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Error, "CRITICAL WARNING"), "CRITICAL WARNING level");
    assert!(contains_colored(&out, Color::Error, "[Timing 38-282]"), "tag is also error color");
}

#[test]
fn vivado_timing_line() {
    let line = "Netlist sorting complete. Time (s): cpu = 00:00:00.11 ; elapsed = 00:00:00.11 . Memory (MB): peak = 10328.246 ; gain = 0.000 ; free physical = 4720 ; free virtual = 57146";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::GetTime, "00:00:00.11"), "cpu time");
    assert!(contains_colored(&out, Color::Size,    "10328.246"),   "peak memory");
    assert!(contains_colored(&out, Color::Numbers, "4720"),        "free physical");
}

#[test]
fn vivado_restored_from_archive() {
    let line = "Restored from archive | CPU: 0.830000 secs | Memory: 16.318893 MB |";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::GoodWord, "Restored from archive"), "label");
    assert!(contains_colored(&out, Color::GetTime,  "0.830000"), "cpu");
    assert!(contains_colored(&out, Color::Size,     "16.318893"), "memory");
}

#[test]
fn vivado_header_separator() {
    let line = "#-----------------------------------------------------------";
    let out = matched(Vivado.process(line));
    assert!(out.contains("\x1b["), "separator has some color code");
}

#[test]
fn vivado_header_kv() {
    let line = "# Process ID         : 337447";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Field, "Process ID"), "key");
    assert!(out.contains("337447"), "value preserved");
}

#[test]
fn vivado_tcl_command() {
    let line = "open_project Projects/driver/driver.xpr";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Keyword, "open_project"), "tcl command");
    assert!(out.contains("Projects/driver/driver.xpr"), "args preserved");
}

#[test]
fn vivado_tcl_set_property() {
    let line = "set_property MAX_FANOUT 32 [get_nets reset]";
    let out = matched(Vivado.process(line));
    assert!(contains_colored(&out, Color::Keyword, "set_property"), "tcl command");
}

#[test]
fn vivado_no_match_on_plain_line() {
    // Plain progress lines with no recognised structure fall through
    assert_no_match(Vivado.process("Reading placement."));
    assert_no_match(Vivado.process("Scanning sources..."));
}

#[test]
fn vivado_pipeline_integration() {
    let plugins = default_plugins();
    let line = "CRITICAL WARNING: [Timing 38-282] The design failed to meet the timing requirements.";
    let out = colorize_line(line, &plugins);
    assert!(contains_colored(&out, Color::Error, "CRITICAL WARNING"), "pipeline vivado critical");
}

// ─── Word colorizer fallback ─────────────────────────────────────────────────

#[test]
fn wordcolor_ip() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("Connection from 192.168.1.1 refused");
    assert!(contains_colored(&out, Color::Host, "192.168.1.1"), "IP colored");
}

#[test]
fn wordcolor_email() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("sent to admin@example.com ok");
    assert!(out.contains(&colorize(Color::Email, "admin@example.com")), "email colored");
}

#[test]
fn wordcolor_bad_word() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("connection failed");
    assert!(contains_colored(&out, Color::BadWord, "failed"), "bad word colored");
}

#[test]
fn wordcolor_good_word() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("service running normally");
    assert!(contains_colored(&out, Color::GoodWord, "running"), "good word colored");
}

#[test]
fn wordcolor_info_uppercase_only() {
    use crate::wordcolor::colorize_words;
    // Plain INFO
    let out = colorize_words("2024-01-01 INFO server started");
    assert!(out.contains(&colorize(Color::Keyword, "INFO")), "INFO should be Keyword color");
    // INFO: with trailing colon (Vivado / common tool log format)
    let out2 = colorize_words("INFO: [Netlist 29-17] Analyzing elements");
    assert!(out2.contains(&colorize(Color::Keyword, "INFO")), "INFO: should highlight INFO");
    // lowercase and mixed-case must not be highlighted
    let out3 = colorize_words("info server started");
    assert!(!out3.contains(&colorize(Color::Keyword, "info")), "info must not be highlighted");
    let out4 = colorize_words("Info server started");
    assert!(!out4.contains(&colorize(Color::Keyword, "Info")), "Info must not be highlighted");
}

#[test]
fn wordcolor_info_in_syslog_message() {
    // INFO inside a syslog message body should be highlighted via word colorization
    let plugins = default_plugins();
    let line = "Jan 15 12:00:00 myhost myapp[123]: INFO: something started";
    let out = colorize_line(line, &plugins);
    assert!(out.contains(&colorize(Color::Keyword, "INFO")), "INFO in syslog msg body");
}

#[test]
fn wordcolor_uri() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("see https://example.com/path for details");
    assert!(out.contains(&colorize(Color::Uri, "https://example.com/path")), "uri colored");
}

#[test]
fn wordcolor_version() {
    use crate::wordcolor::colorize_words;
    let out = colorize_words("upgraded to v3.14.1 successfully");
    assert!(out.contains(&colorize(Color::Version, "v3.14.1")), "version colored");
}

// ─── Full pipeline (colorize_line) ───────────────────────────────────────────

#[test]
fn pipeline_picks_syslog() {
    let plugins = default_plugins();
    let line = "Feb 28 23:59:59 fileserver crond[99]: job started";
    let out = colorize_line(line, &plugins);
    assert!(contains_colored(&out, Color::Process, "crond"), "pipeline → syslog");
}

#[test]
fn pipeline_picks_httpd() {
    let plugins = default_plugins();
    let line = r#"10.0.0.1 - - [01/Jan/2024:00:00:00 +0000] "GET / HTTP/1.1" 200 1024"#;
    let out = colorize_line(line, &plugins);
    assert!(contains_colored(&out, Color::HttpGet, "GET"), "pipeline → httpd");
}

#[test]
fn pipeline_falls_back_to_wordcolor() {
    let plugins = default_plugins();
    let line = "something completely unstructured with error and 192.168.0.1";
    let out = colorize_line(line, &plugins);
    assert!(contains_colored(&out, Color::BadWord, "error"), "fallback bad word");
    assert!(contains_colored(&out, Color::Host, "192.168.0.1"), "fallback ip");
}

#[test]
fn pipeline_ulogd_partial_plus_wordcolor() {
    // A line with netfilter fields but no full-plugin match should have ulogd fields colored
    let plugins = default_plugins();
    let line = "IN=eth0 OUT= SRC=1.2.3.4 DST=5.6.7.8 PROTO=TCP SPT=1000 DPT=443";
    let out = colorize_line(line, &plugins);
    assert!(contains_colored(&out, Color::Field, "SRC"), "partial ulogd");
}
