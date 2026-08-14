/// Semantic color roles matching CCZE's ccze_color_t enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Date,
    Host,
    Process,
    Pid,
    PidBracket,
    Default,
    Email,
    Subject,
    Dir,
    File,
    Size,
    User,
    HttpCodes,
    GetSize,
    HttpGet,
    HttpPost,
    HttpHead,
    HttpPut,
    HttpConnect,
    HttpTrace,
    Unknown,
    GetTime,
    Uri,
    Ident,
    ContentType,
    Error,
    ProxyMiss,
    ProxyHit,
    ProxyDenied,
    ProxyRefresh,
    ProxySwapFail,
    Debug,
    Warning,
    ProxyDirect,
    ProxyParent,
    SwapNum,
    ProxyCreate,
    ProxySwapIn,
    ProxySwapOut,
    ProxyRelease,
    Mac,
    Version,
    Address,
    Numbers,
    Signal,
    Service,
    Protocol,
    BadWord,
    GoodWord,
    SystemWord,
    Incoming,
    Outgoing,
    UniqueId,
    Field,
    Chain,
    Percentage,
    FtpCodes,
    Keyword,
    PkgStatus,
    Package,
    Info,
    String,
}

/// ANSI color codes (foreground)
#[derive(Debug, Clone, Copy)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl AnsiColor {
    fn fg_code(self) -> u8 {
        match self {
            AnsiColor::Black => 30,
            AnsiColor::Red => 31,
            AnsiColor::Green => 32,
            AnsiColor::Yellow => 33,
            AnsiColor::Blue => 34,
            AnsiColor::Magenta => 35,
            AnsiColor::Cyan => 36,
            AnsiColor::White => 37,
            AnsiColor::Default => 39,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fg: AnsiColor,
    pub bold: bool,
}

impl Style {
    pub const fn new(fg: AnsiColor, bold: bool) -> Self {
        Self { fg, bold }
    }

    /// Render `text` wrapped in ANSI escape codes, then reset.
    pub fn apply(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let bold = if self.bold { "1;" } else { "" };
        format!("\x1b[{}{}m{}\x1b[0m", bold, self.fg.fg_code(), text)
    }
}

/// Map a semantic Color to its ANSI Style.
pub fn style_for(color: Color) -> Style {
    use AnsiColor::*;
    match color {
        Color::Date => Style::new(Cyan, false),
        Color::Host => Style::new(Cyan, true),
        Color::Process => Style::new(Green, true),
        Color::Pid => Style::new(Green, false),
        Color::PidBracket => Style::new(Green, false),
        Color::Default => Style::new(Default, false),
        Color::Email => Style::new(Cyan, true),
        Color::Subject => Style::new(Magenta, false),
        Color::Dir => Style::new(Cyan, false),
        Color::File => Style::new(Cyan, false),
        Color::Size => Style::new(Green, false),
        Color::User => Style::new(Yellow, true),
        Color::HttpCodes => Style::new(Yellow, false),
        Color::GetSize => Style::new(Green, false),
        Color::HttpGet => Style::new(Green, true),
        Color::HttpPost => Style::new(Cyan, true),
        Color::HttpHead => Style::new(White, true),
        Color::HttpPut => Style::new(Yellow, true),
        Color::HttpConnect => Style::new(Blue, true),
        Color::HttpTrace => Style::new(Red, true),
        Color::Unknown => Style::new(Default, false),
        Color::GetTime => Style::new(Cyan, false),
        Color::Uri => Style::new(Cyan, true),
        Color::Ident => Style::new(White, true),
        Color::ContentType => Style::new(White, false),
        Color::Error => Style::new(Red, true),
        Color::ProxyMiss => Style::new(Red, false),
        Color::ProxyHit => Style::new(Green, false),
        Color::ProxyDenied => Style::new(Red, true),
        Color::ProxyRefresh => Style::new(Yellow, false),
        Color::ProxySwapFail => Style::new(Red, true),
        Color::Debug => Style::new(Default, false),
        Color::Warning => Style::new(Yellow, true),
        Color::ProxyDirect => Style::new(Cyan, false),
        Color::ProxyParent => Style::new(Blue, false),
        Color::SwapNum => Style::new(Yellow, false),
        Color::ProxyCreate => Style::new(Green, false),
        Color::ProxySwapIn => Style::new(Cyan, false),
        Color::ProxySwapOut => Style::new(Cyan, false),
        Color::ProxyRelease => Style::new(Green, false),
        Color::Mac => Style::new(White, true),
        Color::Version => Style::new(Green, true),
        Color::Address => Style::new(Red, false),
        Color::Numbers => Style::new(Cyan, false),
        Color::Signal => Style::new(Yellow, true),
        Color::Service => Style::new(Magenta, true),
        Color::Protocol => Style::new(Magenta, false),
        Color::BadWord => Style::new(Red, true),
        Color::GoodWord => Style::new(Green, true),
        Color::SystemWord => Style::new(Cyan, true),
        Color::Incoming => Style::new(Green, false),
        Color::Outgoing => Style::new(Yellow, false),
        Color::UniqueId => Style::new(White, false),
        Color::Field => Style::new(Cyan, false),
        Color::Chain => Style::new(White, true),
        Color::Percentage => Style::new(Cyan, false),
        Color::FtpCodes => Style::new(Yellow, false),
        Color::Keyword => Style::new(Yellow, true),
        Color::PkgStatus => Style::new(Cyan, false),
        Color::Package => Style::new(Green, false),
        Color::Info => Style::new(Cyan, true),
        Color::String => Style::new(Green, false),
    }
}

/// Colorize `text` using the semantic color role.
#[inline]
pub fn colorize(color: Color, text: &str) -> String {
    style_for(color).apply(text)
}
