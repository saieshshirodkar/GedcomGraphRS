use crate::config::Gender;
use crate::engine::graph::Graph;
use crate::lines::segments::LineKind;
use crate::render::font_draw::{Fonts, Style};
use crate::render::svg_fmt::{push_alpha, push_esc, push_fx0, push_fx1, push_hex};
use crate::render::theme::{
    COLOR_BACK_LINES, COLOR_BG, COLOR_DUP_DEFAULT, COLOR_FEMALE, COLOR_HEARTH, COLOR_LINES,
    COLOR_MALE, COLOR_UNDEFINED, Scale,
};

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ink {
    pub fill: [u8; 4],
    pub family: &'static str,
}

pub(crate) fn push_text_el(
    o: &mut String,
    x: f32,
    baseline: f32,
    px: f32,
    style: Style,
    ink: Ink,
    body: &str,
) {
    let fill = ink.fill;
    let family = ink.family;
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
    o.reserve(96 + body.len());
    o.push_str("<text x=\"");
    push_fx1(o, x);
    o.push_str("\" y=\"");
    push_fx1(o, baseline);
    o.push_str("\" font-family=\"");
    o.push_str(family);
    o.push_str("\" font-size=\"");
    push_fx0(o, px);
    o.push('"');
    o.push_str(weight);
    o.push_str(slant);
    o.push_str(" fill=\"");
    push_hex(o, fill);
    o.push('"');
    push_alpha(o, fill);
    o.push_str(" text-anchor=\"middle\">");
    push_esc(o, body);
    o.push_str("</text>\n");
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
    let a = &graph.anim;
    let cap = 256 + a.person_count() * 320 + (a.lines.len() + a.back_lines.len()) * 120;
    let mut s = String::with_capacity(cap);
    let _ = std::fmt::Write::write_fmt(
        &mut s,
        format_args!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n<rect width=\"{w}\" height=\"{h}\" fill=\""
        ),
    );
    push_hex(&mut s, COLOR_BG);
    s.push_str("\"/>\n");
    let family = crate::render::embed::face_name(fonts).unwrap_or("sans-serif");
    if let Some(style) = crate::render::embed::font_face_style(fonts) {
        s.push_str(&style);
    }
    s.push_str("<g>\n");

    for l in &a.lines {
        let (x1, y1, x2, y2) = (
            sc.sf(l.x1) + ox,
            sc.sf(l.y1) + oy,
            sc.sf(l.x2) + ox,
            sc.sf(l.y2) + oy,
        );
        if matches!(l.kind, LineKind::Curve { .. }) {
            s.push_str("<path d=\"M");
            push_fx1(&mut s, x1);
            s.push(' ');
            push_fx1(&mut s, y1);
            s.push(' ');
            s.push('C');
            push_fx1(&mut s, x1);
            s.push(' ');
            push_fx1(&mut s, y2);
            s.push(' ');
            push_fx1(&mut s, x2);
            s.push(' ');
            push_fx1(&mut s, y1);
            s.push(' ');
            push_fx1(&mut s, x2);
            s.push(' ');
            push_fx1(&mut s, y2);
        } else {
            s.push_str("<path d=\"M");
            push_fx1(&mut s, x1);
            s.push(' ');
            push_fx1(&mut s, y1);
            s.push(' ');
            s.push('L');
            push_fx1(&mut s, x2);
            s.push(' ');
            push_fx1(&mut s, y2);
        }
        s.push_str("\" stroke=\"");
        push_hex(&mut s, COLOR_LINES);
        s.push_str("\" stroke-width=\"");
        push_fx1(&mut s, 2.0 * sc.s);
        s.push_str("\" fill=\"none\"/>\n");
    }

    for l in &a.back_lines {
        s.push_str("<path d=\"M");
        push_fx1(&mut s, sc.si(l.x1) as f32 + ox);
        s.push(' ');
        push_fx1(&mut s, sc.si(l.y1) as f32 + oy);
        s.push(' ');
        s.push('L');
        push_fx1(&mut s, sc.si(l.x2) as f32 + ox);
        s.push(' ');
        push_fx1(&mut s, sc.si(l.y2) as f32 + oy);
        s.push_str("\" stroke=\"");
        push_hex(&mut s, COLOR_BACK_LINES);
        s.push_str("\" stroke-width=\"");
        push_fx1(&mut s, 1.5 * sc.s);
        s.push_str("\" stroke-dasharray=\"");
        push_fx1(&mut s, 4.0 * sc.s);
        s.push(' ');
        push_fx1(&mut s, 4.0 * sc.s);
        s.push_str("\" fill=\"none\"/>\n");
    }

    for owner in &a.bond_owners {
        let Some(f) = a.families.get(*owner as usize) else {
            continue;
        };
        let Some(b) = &f.bond else { continue };
        let dated = b.marriage_date.is_some();
        let d = if dated || b.w >= 20.0 { 8.0 } else { 6.0 } * sc.s;
        s.push_str("<circle cx=\"");
        push_fx1(&mut s, sc.sf(b.x) + ox + sc.sf(b.w) / 2.0);
        s.push_str("\" cy=\"");
        push_fx1(&mut s, sc.sf(b.y) + oy + sc.sf(b.h) / 2.0);
        s.push_str("\" r=\"");
        push_fx1(&mut s, d / 2.0);
        s.push_str("\" fill=\"");
        push_hex(&mut s, COLOR_HEARTH);
        s.push_str("\"/>\n");
    }

    for dl in &a.duplicate_lines {
        let c = match dl.gender {
            Gender::Male => COLOR_MALE,
            Gender::Female => COLOR_FEMALE,
            _ => COLOR_DUP_DEFAULT,
        };
        s.push_str("<path d=\"M");
        push_fx1(&mut s, sc.sf(dl.x1) + ox);
        s.push(' ');
        push_fx1(&mut s, sc.sf(dl.y1) + oy);
        s.push(' ');
        s.push('Q');
        push_fx1(&mut s, sc.sf(dl.x3) + ox);
        s.push(' ');
        push_fx1(&mut s, sc.sf(dl.y3) + oy);
        s.push(' ');
        push_fx1(&mut s, sc.sf(dl.x2) + ox);
        s.push(' ');
        push_fx1(&mut s, sc.sf(dl.y2) + oy);
        s.push_str("\" stroke=\"");
        push_hex(&mut s, c);
        s.push_str("\" stroke-width=\"");
        push_fx1(&mut s, 2.0 * sc.s);
        s.push_str("\" stroke-linecap=\"round\" fill=\"none\"/>\n");
    }

    super::svg_cards::draw_cards(&mut s, graph, fonts, sc, ox, oy, family);
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
        for (id, name, sex) in [
            ("I1", "A", Some("M")),
            ("I2", "B", Some("F")),
            ("I3", "C", None),
        ] {
            let mut facts = Vec::new();
            if let Some(x) = sex {
                facts.push(Fact::of("SEX", Some(x), Option::None));
            }
            g.push_person(GPerson {
                id: id.to_string(),
                names: vec![Name::of(name)],
                facts,
                parent_fams: Vec::new(),
                spouse_fams: Vec::new(),
                ..GPerson::default()
            });
        }
        g.reindex();
        g
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
        let fonts = Fonts::empty();
        assert_eq!(
            trunc(&fonts, "Hi".to_string(), 1000.0, 68.0, Style::Bold),
            "Hi"
        );
    }

    #[test]
    fn text_el_shape() {
        let mut el = String::new();
        push_text_el(
            &mut el,
            10.0,
            20.0,
            68.0,
            Style::Bold,
            Ink {
                fill: COLOR_TEXT,
                family: "gedcomgraph",
            },
            "Hi",
        );
        assert!(el.contains("font-weight=\"bold\""));
        assert!(el.contains("text-anchor=\"middle\""));
        assert!(el.contains(">Hi</text>"));
        assert!(el.contains("font-family=\"gedcomgraph\""));
        let mut plain = String::new();
        push_text_el(
            &mut plain,
            0.0,
            0.0,
            56.0,
            Style::Regular,
            Ink {
                fill: COLOR_TEXT,
                family: "sans-serif",
            },
            "Yo",
        );
        assert!(!plain.contains("font-weight"));
        let mut esc = String::new();
        push_text_el(
            &mut esc,
            0.0,
            0.0,
            56.0,
            Style::Italic,
            Ink {
                fill: COLOR_TEXT,
                family: "sans-serif",
            },
            "a&b<c>d",
        );
        assert!(esc.contains(">a&amp;b&lt;c&gt;d</text>"));
        assert!(esc.contains("font-style=\"italic\""));
    }
}
