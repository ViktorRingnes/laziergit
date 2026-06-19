use crate::status::FileEntry;
use ratatui::style::Color;

pub const BASE: Color = Color::Rgb(30, 30, 46);
pub const SURFACE: Color = Color::Rgb(49, 50, 68);
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const MUTED: Color = Color::Rgb(166, 173, 200);
pub const OVERLAY: Color = Color::Rgb(108, 112, 134);
pub const CONTEXT_FG: Color = Color::Rgb(166, 173, 200);

pub const ACCENT: Color = Color::Rgb(203, 166, 247);
pub const SELECTION_BG: Color = Color::Rgb(69, 71, 90);
pub const FOLDER: Color = Color::Rgb(137, 180, 250);

pub const RED: Color = Color::Rgb(243, 139, 168);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);

pub const DEL_BG: Color = Color::Rgb(58, 32, 42);
pub const ADD_BG: Color = Color::Rgb(33, 52, 38);
pub const DEL_FG: Color = Color::Rgb(243, 168, 184);
pub const ADD_FG: Color = Color::Rgb(176, 227, 178);
pub const DEL_GUTTER_FG: Color = Color::Rgb(243, 139, 168);
pub const ADD_GUTTER_FG: Color = Color::Rgb(166, 227, 161);
pub const FILLER_BG: Color = Color::Rgb(24, 24, 37);

pub const GUTTER_FG: Color = Color::Rgb(108, 112, 134);
pub const GUTTER_SEP_FG: Color = Color::Rgb(69, 71, 90);
pub const HUNK_FG: Color = Color::Rgb(137, 180, 250);
pub const HUNK_BG: Color = Color::Rgb(30, 37, 46);
pub const GUIDE: Color = Color::Rgb(69, 71, 90);
pub const HEADER_BG: Color = Color::Rgb(24, 24, 37);
pub const BORDER: Color = Color::Rgb(69, 71, 90);
pub const BORDER_FOCUS: Color = Color::Rgb(203, 166, 247);

pub struct Guides {
    pub vertical: &'static str,
    pub branch: &'static str,
    pub last: &'static str,
    pub blank: &'static str,
}

pub struct Theme {
    pub nerd_fonts: bool,
    pub guides_ascii: bool,
}

impl Theme {
    pub fn detect() -> Self {
        let ascii = std::env::var_os("LAZIERGIT_ASCII").is_some()
            || std::env::var("TERM").map(|t| t == "linux").unwrap_or(false);
        Theme {
            nerd_fonts: !ascii,
            guides_ascii: ascii,
        }
    }

    pub fn guides(&self) -> Guides {
        if self.guides_ascii {
            Guides {
                vertical: "|   ",
                branch: "|-- ",
                last: "`-- ",
                blank: "    ",
            }
        } else {
            Guides {
                vertical: "│   ",
                branch: "├── ",
                last: "└── ",
                blank: "    ",
            }
        }
    }
}

pub fn folder_glyph(open: bool) -> char {
    if open { '▾' } else { '▸' }
}

pub fn folder_icon(open: bool) -> char {
    if open { '\u{f07c}' } else { '\u{f07b}' }
}

pub fn status_color(file: &FileEntry) -> Color {
    if file.staged { GREEN } else { RED }
}

pub fn icon_for(path: &str) -> (char, Color) {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => ('\u{e7a8}', Color::Rgb(222, 108, 58)),
        "toml" | "lock" => ('\u{e615}', MUTED),
        "md" => ('\u{e73e}', TEXT),
        "json" | "yaml" | "yml" => ('\u{e60b}', YELLOW),
        "js" => ('\u{e60c}', YELLOW),
        "ts" => ('\u{e628}', FOLDER),
        "py" => ('\u{e606}', YELLOW),
        "go" => ('\u{e627}', Color::Rgb(116, 199, 236)),
        "c" | "h" => ('\u{e61e}', FOLDER),
        "cpp" | "hpp" | "cc" => ('\u{e61d}', FOLDER),
        "sh" | "bash" | "zsh" => ('\u{e795}', GREEN),
        "css" => ('\u{e749}', FOLDER),
        "html" => ('\u{e736}', Color::Rgb(222, 108, 58)),
        "png" | "jpg" | "jpeg" | "gif" | "svg" => ('\u{e60d}', ACCENT),
        "gitignore" | "gitattributes" | "gitmodules" => ('\u{e702}', Color::Rgb(222, 108, 58)),
        _ => ('\u{e64e}', OVERLAY),
    }
}
