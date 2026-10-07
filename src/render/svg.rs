use crate::config::Gender;
use crate::engine::graph::Graph;
use crate::lines::segments::LineKind;
use crate::render::font_draw::{Fonts, Style};
use crate::render::theme::{
    COLOR_BACK_LINES, COLOR_BG, COLOR_DUP_DEFAULT, COLOR_FEMALE, COLOR_HEARTH, COLOR_LINES,
    COLOR_MALE, COLOR_UNDEFINED, Scale,
};

#[inline]
fn rhe(x: f64) -> f64 {
    let f = x.floor();
    let d = x - f;
    let up = if d > 0.5 {
        true
    } else if d < 0.5 {
        false
    } else {
        (f as i64) % 2 != 0
    };
    if up { f + 1.0 } else { f }
}

fn push_u64(s: &mut String, mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    for b in &buf[i..] {
        s.push(char::from(*b));
    }
}

fn scaled(v: f32, mul: f64) -> Option<i64> {
    if !v.is_finite() || v.abs() > 1.0e6 {
        return Option::None;
    }
    Some(rhe(f64::from(v) * mul) as i64)
}

pub(crate) fn fx1(v: f32) -> String {
    let Some(i) = scaled(v, 10.0) else {
        return format!("{v:.1}");
    };
    let mut s = String::with_capacity(12);
    if i < 0 {
        s.push('-');
    }
    let m = i.unsigned_abs();
    push_u64(&mut s, m / 10);
    s.push('.');
    s.push(char::from(b'0' + (m % 10) as u8));
    if i == 0 && v.is_sign_negative() {
        s.insert(0, '-');
    }
    s
}

pub(crate) fn fx0(v: f32) -> String {
    let Some(i) = scaled(v, 1.0) else {
        return format!("{v:.0}");
    };
    let mut s = String::with_capacity(12);
    if i < 0 {
        s.push('-');
    }
    push_u64(&mut s, i.unsigned_abs());
    if i == 0 && v.is_sign_negative() {
        s.insert(0, '-');
    }
    s
}

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
    let mut o = String::with_capacity(96 + body.len());
    o.push_str("<text x=\"");
    o.push_str(&fx1(x));
    o.push_str("\" y=\"");
    o.push_str(&fx1(baseline));
    o.push_str("\" font-family=\"sans-serif\" font-size=\"");
    o.push_str(&fx0(px));
    o.push('"');
    o.push_str(weight);
    o.push_str(slant);
    o.push_str(" fill=\"");
    o.push_str(&hex(fill));
    o.push('"');
    o.push_str(&alpha(fill));
    o.push_str(" text-anchor=\"middle\">");
    o.push_str(&esc(body));
    o.push_str("</text>\n");
    o
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
            s.push_str("<path d=\"M");
            s.push_str(&fx1(x1));
            s.push(' ');
            s.push_str(&fx1(y1));
            s.push(' ');
            s.push('C');
            s.push_str(&fx1(x1));
            s.push(' ');
            s.push_str(&fx1(y2));
            s.push(' ');
            s.push_str(&fx1(x2));
            s.push(' ');
            s.push_str(&fx1(y1));
            s.push(' ');
            s.push_str(&fx1(x2));
            s.push(' ');
            s.push_str(&fx1(y2));
            s.push_str("\" stroke=\"");
            s.push_str(&hex(COLOR_LINES));
            s.push_str("\" stroke-width=\"");
            s.push_str(&fx1(2.0 * sc.s));
            s.push_str("\" fill=\"none\"/>\n");
        } else {
            s.push_str("<path d=\"M");
            s.push_str(&fx1(x1));
            s.push(' ');
            s.push_str(&fx1(y1));
            s.push(' ');
            s.push('L');
            s.push_str(&fx1(x2));
            s.push(' ');
            s.push_str(&fx1(y2));
            s.push_str("\" stroke=\"");
            s.push_str(&hex(COLOR_LINES));
            s.push_str("\" stroke-width=\"");
            s.push_str(&fx1(2.0 * sc.s));
            s.push_str("\" fill=\"none\"/>\n");
        }
    }
    for l in &graph.anim.back_lines {
        s.push_str("<path d=\"M");
        s.push_str(&fx1(sc.si(l.x1) as f32 + ox));
        s.push(' ');
        s.push_str(&fx1(sc.si(l.y1) as f32 + oy));
        s.push(' ');
        s.push('L');
        s.push_str(&fx1(sc.si(l.x2) as f32 + ox));
        s.push(' ');
        s.push_str(&fx1(sc.si(l.y2) as f32 + oy));
        s.push_str("\" stroke=\"");
        s.push_str(&hex(COLOR_BACK_LINES));
        s.push_str("\" stroke-width=\"");
        s.push_str(&fx1(1.5 * sc.s));
        s.push_str("\" stroke-dasharray=\"");
        s.push_str(&fx1(4.0 * sc.s));
        s.push(' ');
        s.push_str(&fx1(4.0 * sc.s));
        s.push_str("\" fill=\"none\"/>\n");
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
        s.push_str("<circle cx=\"");
        s.push_str(&fx1(bx + sc.sf(b.w) / 2.0));
        s.push_str("\" cy=\"");
        s.push_str(&fx1(by + sc.sf(b.h) / 2.0));
        s.push_str("\" r=\"");
        s.push_str(&fx1(d / 2.0));
        s.push_str("\" fill=\"");
        s.push_str(&hex(COLOR_HEARTH));
        s.push_str("\"/>\n");
    }
    for dl in &graph.anim.duplicate_lines {
        let c = match dl.gender {
            Gender::Male => COLOR_MALE,
            Gender::Female => COLOR_FEMALE,
            _ => COLOR_DUP_DEFAULT,
        };
        s.push_str("<path d=\"M");
        s.push_str(&fx1(sc.sf(dl.x1) + ox));
        s.push(' ');
        s.push_str(&fx1(sc.sf(dl.y1) + oy));
        s.push(' ');
        s.push('Q');
        s.push_str(&fx1(sc.sf(dl.x3) + ox));
        s.push(' ');
        s.push_str(&fx1(sc.sf(dl.y3) + oy));
        s.push(' ');
        s.push_str(&fx1(sc.sf(dl.x2) + ox));
        s.push(' ');
        s.push_str(&fx1(sc.sf(dl.y2) + oy));
        s.push_str("\" stroke=\"");
        s.push_str(&hex(c));
        s.push_str("\" stroke-width=\"");
        s.push_str(&fx1(2.0 * sc.s));
        s.push_str("\" stroke-linecap=\"round\" fill=\"none\"/>\n");
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
    fn fx1_matches_std_format() {
        let mut s: u32 = 22222;
        let mut checked = 0u32;
        for _ in 0..300_000 {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let v = f32::from_bits(s);
            if !v.is_finite() || v.abs() > 1.0e6 {
                continue;
            }
            checked += 1;
            assert_eq!(fx1(v), format!("{v:.1}"), "fx1 mismatch on {v:?}");
            assert_eq!(fx0(v), format!("{v:.0}"), "fx0 mismatch on {v:?}");
        }
        assert!(checked > 100_000, "sample too small: {checked}");
    }

    #[test]
    fn fx1_matches_std_on_dense_grid() {
        for i in -200_000i64..200_000 {
            let v = (i as f64 * 0.0007) as f32;
            assert_eq!(fx1(v), format!("{v:.1}"), "fx1 grid mismatch on {v:?}");
            assert_eq!(fx0(v), format!("{v:.0}"), "fx0 grid mismatch on {v:?}");
        }
    }

    #[test]
    fn fx_handles_edges() {
        for v in [
            0.0f32,
            -0.0,
            0.05,
            -0.05,
            0.25,
            0.35,
            1.05,
            2.5,
            -0.25,
            1e30,
            -1e30,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ] {
            assert_eq!(fx1(v), format!("{v:.1}"), "edge fx1 on {v:?}");
            assert_eq!(fx0(v), format!("{v:.0}"), "edge fx0 on {v:?}");
        }
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
