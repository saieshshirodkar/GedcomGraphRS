use crate::render::font_file::Font;

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

fn try_load(names: &[&str]) -> Option<Font> {
    for dir in [
        "/usr/share/fonts/TTF",
        "/usr/share/fonts/liberation",
        "/usr/share/fonts/noto",
    ] {
        for n in names {
            let p = format!("{dir}/{n}");
            if let Some(f) = Font::load(&p) {
                return Some(f);
            }
        }
    }
    Option::None
}

impl Fonts {
    pub fn load() -> Fonts {
        Fonts {
            regular: try_load(&[
                "DejaVuSans.ttf",
                "LiberationSans-Regular.ttf",
                "NotoSans-Regular.ttf",
            ]),
            bold: try_load(&[
                "DejaVuSans-Bold.ttf",
                "LiberationSans-Bold.ttf",
                "NotoSans-Bold.ttf",
            ]),
            italic: try_load(&[
                "DejaVuSans-Oblique.ttf",
                "LiberationSans-Italic.ttf",
                "NotoSans-Italic.ttf",
            ]),
            fallback: try_load(&["DejaVuSans.ttf", "NotoSans-Regular.ttf"]),
        }
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
        if let Some(f) = self.regular.as_ref() {
            return Some((f, 0));
        }
        Option::None
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

    fn empty() -> Fonts {
        Fonts {
            regular: Option::None,
            bold: Option::None,
            italic: Option::None,
            fallback: Option::None,
        }
    }

    #[test]
    fn empty_fonts_measure_zero() {
        let f = empty();
        assert_eq!(f.measure("Hello", 68.0, Style::Bold), 0.0);
        assert_eq!(f.measure("", 68.0, Style::Regular), 0.0);
        assert_eq!(f.measure("Hi", 56.0, Style::Italic), 0.0);
    }

    #[test]
    fn loaded_fonts_measure_positive() {
        let f = Fonts::load();
        if f.regular.is_none() {
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
}
