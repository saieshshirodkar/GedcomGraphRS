use crate::engine::graph::Graph;
use crate::render::font_draw::{Fonts, Style};
use crate::render::svg::{alpha, border_of, fx1, hex, text_el, trunc};
use crate::render::texts::{date_lines, is_dead, name_lines, titles_of};
use crate::render::theme::{
    COLOR_BACK_ELEMENT, COLOR_PARTNER, COLOR_RIBBON_BLACK, COLOR_RIBBON_WHITE, COLOR_TEXT,
    COLOR_TEXT_VEILED, Scale,
};

struct Geom {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

fn open_rect(s: &mut String, g: &Geom, rx: f32) {
    s.push_str("<rect x=\"");
    s.push_str(&fx1(g.x));
    s.push_str("\" y=\"");
    s.push_str(&fx1(g.y));
    s.push_str("\" width=\"");
    s.push_str(&fx1(g.w));
    s.push_str("\" height=\"");
    s.push_str(&fx1(g.h));
    s.push_str("\" rx=\"");
    s.push_str(&fx1(rx));
    s.push('"');
}

fn card_rects(
    s: &mut String,
    g: &Geom,
    bg_corner: f32,
    corner: f32,
    bg: [u8; 4],
    border: [u8; 4],
    bw: f32,
) {
    open_rect(s, g, bg_corner);
    s.push_str(" fill=\"");
    s.push_str(&hex(bg));
    s.push('"');
    s.push_str(&alpha(bg));
    s.push_str("/>\n");
    open_rect(s, g, corner);
    s.push_str(" fill=\"none\" stroke=\"");
    s.push_str(&hex(border));
    s.push_str("\" stroke-width=\"");
    s.push_str(&fx1(bw));
    s.push_str("\"/>\n");
}

pub(crate) fn draw_cards(
    s: &mut String,
    graph: &Graph,
    fonts: &Fonts,
    sc: &Scale,
    ox: f32,
    oy: f32,
) {
    for pid in 0..graph.anim.person_count() as u32 {
        let p = &graph.anim.persons[pid as usize];
        let x = sc.si(p.base.x) as f32 + ox;
        let y = sc.si(p.base.y) as f32 + oy;
        let w = sc.si(p.base.w) as f32;
        let h = sc.si(p.base.h) as f32;
        let border = border_of(&graph.gedcom, p.person);
        let bg = if p.base.mini {
            COLOR_BACK_ELEMENT
        } else if p.acquired {
            COLOR_PARTNER
        } else {
            COLOR_BACK_ELEMENT
        };
        let g = Geom { x, y, w, h };
        card_rects(s, &g, sc.bg_corner, sc.corner, bg, border, sc.border);
        if p.base.mini {
            let txt = if p.amount > 100 {
                "100+".to_string()
            } else {
                p.amount.to_string()
            };
            s.push_str(&text_el(
                x + w / 2.0,
                y + (h + sc.mini_px * 0.7) / 2.0,
                sc.mini_px,
                Style::Bold,
                COLOR_TEXT,
                &txt,
            ));
            continue;
        }
        if is_dead(&graph.gedcom, p.person) {
            let rs = sc.ribbon / 22.0;
            let rx = x + w - sc.ribbon;
            s.push_str(&format!(
                "<polygon points=\"{rx:.1},{y:.1} {:.1},{y:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{}\"/>\n",
                rx + 8.0 * rs,
                rx + 22.0 * rs,
                y + 14.0 * rs,
                rx + 22.0 * rs,
                y + 22.0 * rs,
                hex(COLOR_RIBBON_WHITE)
            ));
            s.push_str(&format!(
                "<polygon points=\"{:.1},{y:.1} {:.1},{y:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{}\"/>\n",
                rx + 2.0 * rs,
                rx + 6.0 * rs,
                rx + 22.0 * rs,
                y + 16.0 * rs,
                rx + 22.0 * rs,
                y + 20.0 * rs,
                hex(COLOR_RIBBON_BLACK)
            ));
        }
        let max_w = w - sc.pad_x * 2.0 + 2.0;
        let mut cy = y + sc.pad_top;
        for name in name_lines(&graph.gedcom, p.person, p.duplicate) {
            let t = trunc(fonts, name, max_w, sc.name_px, Style::Bold);
            s.push_str(&text_el(
                x + w / 2.0,
                cy + sc.name_px * 0.8,
                sc.name_px,
                Style::Bold,
                COLOR_TEXT,
                &t,
            ));
            cy += sc.lh_name;
        }
        let titles = titles_of(&graph.gedcom, p.person);
        if !titles.is_empty() {
            for line in titles.split('\n') {
                let t = trunc(fonts, line.to_string(), max_w, sc.title_px, Style::Italic);
                s.push_str(&text_el(
                    x + w / 2.0,
                    cy + sc.title_px * 0.8,
                    sc.title_px,
                    Style::Italic,
                    COLOR_TEXT,
                    &t,
                ));
                cy += sc.lh_title;
            }
        }
        for line in date_lines(&graph.gedcom, p.person) {
            let t = trunc(fonts, line, max_w, sc.date_px, Style::Regular);
            s.push_str(&text_el(
                x + w / 2.0,
                cy + sc.date_px * 0.8,
                sc.date_px,
                Style::Regular,
                COLOR_TEXT_VEILED,
                &t,
            ));
            cy += sc.lh_date;
        }
    }
}
