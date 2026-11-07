use crate::render::font_file::Font;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Regular,
    Bold,
    Italic,
}

pub struct Fonts {
    pub regular: Option<Font>,
    pub bold: Option<Font>,
    pub italic: Option<Font>,
    pub fallback: Option<Font>,
}

const REGULAR_STEMS: &[&str] = &[
    "DejaVuSans",
    "LiberationSans-Regular",
    "NotoSans-Regular",
    "FreeSans",
    "Roboto-Regular",
    "NotoSans",
    "Arial",
    "ArialMT",
    "Helvetica",
    "Verdana",
    "Tahoma",
];

const BOLD_STEMS: &[&str] = &[
    "DejaVuSans-Bold",
    "LiberationSans-Bold",
    "NotoSans-Bold",
    "FreeSansBold",
    "Roboto-Bold",
    "Arial-Bold",
    "Arial_Bold",
    "Arial-BoldMT",
    "Helvetica-Bold",
    "Verdana-Bold",
];

const ITALIC_STEMS: &[&str] = &[
    "DejaVuSans-Oblique",
    "LiberationSans-Italic",
    "NotoSans-Italic",
    "FreeSansOblique",
    "Roboto-Italic",
    "Arial-Italic",
    "Arial_Italic",
    "Verdana-Italic",
];

const EXTS: &[&str] = &["ttf", "ttc", "otf"];

const SCAN_BUDGET: usize = 4000;
const MAX_DEPTH: usize = 4;

#[cfg(target_os = "macos")]
const DIRS: &[&str] = &[
    "/System/Library/Fonts",
    "/System/Library/Fonts/Supplemental",
    "/Library/Fonts",
    "/System/Library/AssetsV2/com_apple_MobileAsset_Font7",
];

#[cfg(target_os = "windows")]
const DIRS: &[&str] = &["C:\\Windows\\Fonts", "C:\\Windows\\Fonts\\truetype"];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const DIRS: &[&str] = &[
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    "/usr/share/fonts/TTF",
    "/usr/share/fonts/liberation",
    "/usr/share/fonts/liberation2",
    "/usr/share/fonts/noto",
    "/usr/share/fonts/dejavu",
    "/usr/share/fonts/TTF/DejaVu",
];

fn wanted(name: &std::ffi::OsStr, stems: &[&str]) -> bool {
    let Some(raw) = name.to_str() else {
        return false;
    };
    let mut parts = raw.rsplitn(2, '.');
    let ext = parts.next().unwrap_or_default().to_ascii_lowercase();
    if !EXTS.contains(&ext.as_str()) {
        return false;
    }
    let stem = match parts.next() {
        Some(s) => s.rsplit_once('.').map_or(s, |(a, _)| a),
        None => raw,
    };
    stems.iter().any(|w| w.eq_ignore_ascii_case(stem))
}

fn collect(dir: &Path, stems: &[&str], out: &mut Vec<PathBuf>, budget: &mut usize, depth: usize) {
    if *budget == 0 || depth > MAX_DEPTH {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut subdirs: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        if *budget == 0 {
            return;
        }
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            subdirs.push(e.path());
        } else if wanted(&e.file_name(), stems) {
            out.push(e.path());
            *budget -= 1;
        }
    }
    for d in subdirs {
        collect(&d, stems, out, budget, depth + 1);
    }
}

fn candidates(stems: &[&str]) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut budget = SCAN_BUDGET;
    for d in DIRS {
        collect(Path::new(d), stems, &mut out, &mut budget, 0);
        if out.len() >= 6 {
            break;
        }
    }
    out
}

fn try_load(stems: &[&str]) -> Option<Font> {
    for p in candidates(stems) {
        let Some(name) = p.to_str() else { continue };
        if let Some(f) = Font::load(name) {
            return Some(f);
        }
    }
    Option::None
}

impl Fonts {
    pub fn load() -> Fonts {
        let regular = try_load(REGULAR_STEMS);
        Fonts {
            bold: try_load(BOLD_STEMS).or_else(|| regular.clone()),
            italic: try_load(ITALIC_STEMS).or_else(|| regular.clone()),
            fallback: regular.clone(),
            regular,
        }
    }

    pub fn empty() -> Fonts {
        Fonts {
            regular: Option::None,
            bold: Option::None,
            italic: Option::None,
            fallback: Option::None,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.regular.is_some()
    }

    fn face(&self, style: Style) -> Option<&Font> {
        match style {
            Style::Regular => self.regular.as_ref(),
            Style::Bold => self.bold.as_ref().or(self.regular.as_ref()),
            Style::Italic => self.italic.as_ref().or(self.regular.as_ref()),
        }
    }

    fn glyph(&self, style: Style, c: char) -> Option<(&Font, u16)> {
        if let Some(f) = self.face(style) {
            let g = f.glyph_id(c);
            if g != 0 {
                return Some((f, g));
            }
        }
        if let Some(f) = self.fallback.as_ref() {
            let g = f.glyph_id(c);
            if g != 0 {
                return Some((f, g));
            }
        }
        self.regular.as_ref().map(|f| (f, 0))
    }

    pub fn measure(&self, text: &str, px: f32, style: Style) -> f32 {
        let mut w = 0.0f32;
        for c in text.chars() {
            if let Some((f, g)) = self.glyph(style, c) {
                w += f.advance(g) * px / f.units;
            }
        }
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_fonts_measure_zero() {
        let f = Fonts::empty();
        assert!(!f.is_usable());
        assert_eq!(f.measure("Hello", 68.0, Style::Bold), 0.0);
        assert_eq!(f.measure("", 68.0, Style::Regular), 0.0);
        assert_eq!(f.measure("Hi", 56.0, Style::Italic), 0.0);
    }

    #[test]
    fn loaded_fonts_measure_positive() {
        let f = Fonts::load();
        if !f.is_usable() {
            return;
        }
        assert!(f.measure("Ag", 68.0, Style::Regular) > 0.0);
        assert!(f.measure("Ag", 68.0, Style::Bold) > 0.0);
        assert!(f.measure("Ag", 68.0, Style::Italic) > 0.0);
        assert!(
            f.measure("A longer line", 56.0, Style::Regular)
                > f.measure("Ag", 56.0, Style::Regular)
        );
    }

    #[test]
    fn stem_matching_is_case_and_ext_aware() {
        let p = std::ffi::OsStr::new("DejaVuSans.ttf");
        assert!(wanted(p, REGULAR_STEMS));
        assert!(!wanted(p, BOLD_STEMS));
        let b = std::ffi::OsStr::new("DejaVuSans-Bold.TTF");
        assert!(wanted(b, BOLD_STEMS));
        assert!(!wanted(std::ffi::OsStr::new("notes.txt"), REGULAR_STEMS));
        assert!(!wanted(std::ffi::OsStr::new("noext"), REGULAR_STEMS));
    }

    #[test]
    fn candidates_find_something_or_none_without_panic() {
        let c = candidates(REGULAR_STEMS);
        assert!(c.len() < SCAN_BUDGET + 1);
    }

    #[test]
    fn fallback_covers_missing_styles() {
        let f = Fonts::load();
        if !f.is_usable() {
            return;
        }
        assert!(f.bold.is_some());
        assert!(f.italic.is_some());
        assert!(f.fallback.is_some());
    }
}
