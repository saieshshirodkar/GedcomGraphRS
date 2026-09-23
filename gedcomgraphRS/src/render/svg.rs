use crate::config::Gender;
use crate::engine::graph::Graph;
use crate::lines::segments::LineKind;
use crate::render::font_draw::{Fonts, Style};
use crate::render::theme::{
    COLOR_BACK_LINES, COLOR_BG, COLOR_DUP_DEFAULT, COLOR_FEMALE, COLOR_HEARTH, COLOR_LINES,
    COLOR_MALE, COLOR_UNDEFINED, Scale,
};

pub(crate) fn hex(c: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

pub(crate) fn alpha(c: [u8; 4]) -> String {
    if c[3] == 255 {
        String::new()
    } else {
        format!(" fill-opacity=\"{:.3}\"", c[3] as f32 / 255.0)
    }
}

pub(crate) fn esc(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            _ => o.push(c),
        }
    }
    o
}

pub(crate) fn border_of(ged: &crate::model::gedcom::GedcomData, person: u32) -> [u8; 4] {
    match Gender::of_person(ged, person) {
        Gender::Male => COLOR_MALE,
        Gender::Female => COLOR_FEMALE,
        _ => COLOR_UNDEFINED,
    }
}

pub(crate) fn trunc(fonts: &Fonts, mut s: String, max_w: f32, px: f32, style: Style) -> String {
    if fonts.measure(&s, px, style) > max_w {
        while fonts.measure(&(s.clone() + "..."), px, style) > max_w && s.len() > 3 {
            s.pop();
        }
        s.push_str("...");
    }
    s
}

pub(crate) fn text_el(
    x: f32,
    baseline: f32,
    px: f32,
    style: Style,
    fill: [u8; 4],
    body: &str,
) -> String {
    let weight = if style == Style::Bold {
        " font-weight=\"bold\""
    } else {
        ""
    };
    let slant = if style == Style::Italic {
        " font-style=\"italic\""
    } else {
        ""
    };
    format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" font-family=\"sans-serif\" font-size=\"{:.0}\"{}{} fill=\"{}\"{} text-anchor=\"middle\">{}</text>\n",
        x,
        baseline,
        px,
        weight,
        slant,
        hex(fill),
        alpha(fill),
        esc(body)
    )
}

pub fn render_svg(
    graph: &Graph,
    fonts: &Fonts,
    sc: &Scale,
    ox: f32,
    oy: f32,
    w: i32,
    h: i32,
) -> String {
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>\n<g>\n",
        hex(COLOR_BG)
    );
    for l in &graph.anim.lines {
        let x1 = sc.sf(l.x1) + ox;
        let y1 = sc.sf(l.y1) + oy;
        let x2 = sc.sf(l.x2) + ox;
        let y2 = sc.sf(l.y2) + oy;
        if matches!(l.kind, LineKind::Curve { .. }) {
            s.push_str(&format!(
                "<path d=\"M{x1:.1} {y1:.1} C{x1:.1} {y2:.1} {x2:.1} {y1:.1} {x2:.1} {y2:.1}\" stroke=\"{}\" stroke-width=\"{:.1}\" fill=\"none\"/>\n",
                hex(COLOR_LINES),
                2.0 * sc.s
            ));
        } else {
            s.push_str(&format!(
                "<path d=\"M{x1:.1} {y1:.1} L{x2:.1} {y2:.1}\" stroke=\"{}\" stroke-width=\"{:.1}\" fill=\"none\"/>\n",
                hex(COLOR_LINES),
                2.0 * sc.s
            ));
        }
    }
    for l in &graph.anim.back_lines {
        s.push_str(&format!(
            "<path d=\"M{:.1} {:.1} L{:.1} {:.1}\" stroke=\"{}\" stroke-width=\"{:.1}\" stroke-dasharray=\"{:.1} {:.1}\" fill=\"none\"/>\n",
            sc.si(l.x1) as f32 + ox,
            sc.si(l.y1) as f32 + oy,
            sc.si(l.x2) as f32 + ox,
            sc.si(l.y2) as f32 + oy,
            hex(COLOR_BACK_LINES),
            1.5 * sc.s,
            4.0 * sc.s,
            4.0 * sc.s
        ));
    }
    for b in graph.anim.bond_views() {
        let bx = sc.sf(graph.anim.families[b.owner as usize]
            .bond
            .as_ref()
            .map(|q| q.x)
            .unwrap_or(0.0))
            + ox;
        let by = sc.sf(graph.anim.families[b.owner as usize]
            .bond
            .as_ref()
            .map(|q| q.y)
            .unwrap_or(0.0))
            + oy;
        let has_date = b.marriage_date.is_some();
        let d = if has_date || b.w >= 20.0 { 8.0 } else { 6.0 } * sc.s;
        s.push_str(&format!(
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"{}\"/>\n",
            bx + sc.sf(b.w) / 2.0,
            by + sc.sf(b.h) / 2.0,
            d / 2.0,
            hex(COLOR_HEARTH)
        ));
    }
    for dl in &graph.anim.duplicate_lines {
        let c = match dl.gender {
            Gender::Male => COLOR_MALE,
            Gender::Female => COLOR_FEMALE,
            _ => COLOR_DUP_DEFAULT,
        };
        s.push_str(&format!(
            "<path d=\"M{:.1} {:.1} Q{:.1} {:.1} {:.1} {:.1}\" stroke=\"{}\" stroke-width=\"{:.1}\" stroke-linecap=\"round\" fill=\"none\"/>\n",
            sc.sf(dl.x1) + ox,
            sc.sf(dl.y1) + oy,
            sc.sf(dl.x3) + ox,
            sc.sf(dl.y3) + oy,
            sc.sf(dl.x2) + ox,
            sc.sf(dl.y2) + oy,
            hex(c),
            2.0 * sc.s
        ));
    }
    super::svg_cards::draw_cards(&mut s, graph, fonts, sc, ox, oy);
    s.push_str("</g>\n</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GPerson, GedcomData, Name};
    use crate::render::font_draw::Fonts;
    use crate::render::theme::COLOR_TEXT;

    fn ged_two() -> GedcomData {
        let mut g = GedcomData::empty();
        g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("A")],
            facts: vec![Fact::of("SEX", Some("M"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.push_person(GPerson {
            id: "I2".to_string(),
            names: vec![Name::of("B")],
            facts: vec![Fact::of("SEX", Some("F"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.push_person(GPerson {
            id: "I3".to_string(),
            names: vec![Name::of("C")],
            facts: Vec::new(),
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.reindex();
        g
    }

    #[test]
    fn hex_format() {
        assert_eq!(hex([0x44, 0xAA, 0xFF, 255]), "#44AAFF");
        assert_eq!(hex([0, 0, 0, 0]), "#000000");
    }

    #[test]
    fn alpha_opacity() {
        assert_eq!(alpha([255, 255, 255, 255]), "");
        assert_eq!(alpha([0, 0, 0, 0x66]), " fill-opacity=\"0.400\"");
    }

    #[test]
    fn esc_specials() {
        assert_eq!(esc("a&b<c>d"), "a&amp;b&lt;c&gt;d");
        assert_eq!(esc("plain"), "plain");
    }

    #[test]
    fn border_by_gender() {
        let g = ged_two();
        assert_eq!(border_of(&g, 0), COLOR_MALE);
        assert_eq!(border_of(&g, 1), COLOR_FEMALE);
        assert_eq!(border_of(&g, 2), COLOR_UNDEFINED);
        assert_eq!(border_of(&g, 99), COLOR_UNDEFINED);
    }

    #[test]
    fn trunc_keeps_fitting_text() {
        let fonts = Fonts {
            regular: Option::None,
            bold: Option::None,
            italic: Option::None,
            fallback: Option::None,
        };
        assert_eq!(
            trunc(&fonts, "Hi".to_string(), 1000.0, 68.0, Style::Bold),
            "Hi"
        );
    }

    #[test]
    fn text_el_shape() {
        let el = text_el(10.0, 20.0, 68.0, Style::Bold, COLOR_TEXT, "Hi");
        assert!(el.contains("font-weight=\"bold\""));
        assert!(el.contains("text-anchor=\"middle\""));
        assert!(el.contains(">Hi</text>"));
        let plain = text_el(0.0, 0.0, 56.0, Style::Regular, COLOR_TEXT, "Yo");
        assert!(!plain.contains("font-weight"));
    }
}
