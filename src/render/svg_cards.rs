use crate::engine::graph::Graph;
use crate::render::font_draw::{Fonts, Style};
use crate::render::svg::{border_of, push_text_el, trunc};
use crate::render::svg_fmt::{push_alpha, push_fx1, push_hex};
use crate::render::texts::{date_lines, is_dead, name_lines, titles_of};
use crate::render::theme::{
    COLOR_BACK_ELEMENT, COLOR_PARTNER, COLOR_RIBBON_BLACK, COLOR_RIBBON_WHITE, COLOR_TEXT,
    COLOR_TEXT_VEILED, Scale,
};
use std::fmt::Write as _;

struct Geom {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

fn open_rect(s: &mut String, g: &Geom, rx: f32) {
    s.reserve(72);
    s.push_str("<rect x=\"");
    push_fx1(s, g.x);
    s.push_str("\" y=\"");
    push_fx1(s, g.y);
    s.push_str("\" width=\"");
    push_fx1(s, g.w);
    s.push_str("\" height=\"");
    push_fx1(s, g.h);
    s.push_str("\" rx=\"");
    push_fx1(s, rx);
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
    push_hex(s, bg);
    s.push('"');
    push_alpha(s, bg);
    s.push_str("/>\n");
    open_rect(s, g, corner);
    s.push_str(" fill=\"none\" stroke=\"");
    push_hex(s, border);
    s.push_str("\" stroke-width=\"");
    push_fx1(s, bw);
    s.push_str("\"/>\n");
}

fn mini_text(s: &mut String, x: f32, y: f32, h: f32, sc: &Scale, amount: u32) {
    let mut txt = String::with_capacity(4);
    if amount > 100 {
        txt.push_str("100+");
    } else {
        let _ = write!(txt, "{amount}");
    }
    push_text_el(
        s,
        x,
        y + (h + sc.mini_px * 0.7) / 2.0,
        sc.mini_px,
        Style::Bold,
        COLOR_TEXT,
        &txt,
    );
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
        let bg = if p.acquired && !p.base.mini {
            COLOR_PARTNER
        } else {
            COLOR_BACK_ELEMENT
        };
        card_rects(
            s,
            &Geom { x, y, w, h },
            sc.bg_corner,
            sc.corner,
            bg,
            border,
            sc.border,
        );
        if p.base.mini {
            mini_text(s, x + w / 2.0, y, h, sc, p.amount);
            continue;
        }
        if is_dead(&graph.gedcom, p.person) {
            let rs = sc.ribbon / 22.0;
            let rx = x + w - sc.ribbon;
            s.reserve(180);
            s.push_str("<polygon points=\"");
            push_fx1(s, rx);
            s.push(',');
            push_fx1(s, y);
            s.push(' ');
            push_fx1(s, rx + 8.0 * rs);
            s.push(',');
            push_fx1(s, y);
            s.push(' ');
            push_fx1(s, rx + 22.0 * rs);
            s.push(',');
            push_fx1(s, y + 14.0 * rs);
            s.push(' ');
            push_fx1(s, rx + 22.0 * rs);
            s.push(',');
            push_fx1(s, y + 22.0 * rs);
            s.push_str("\" fill=\"");
            push_hex(s, COLOR_RIBBON_WHITE);
            s.push_str("\"/>\n<polygon points=\"");
            push_fx1(s, rx + 2.0 * rs);
            s.push(',');
            push_fx1(s, y);
            s.push(' ');
            push_fx1(s, rx + 6.0 * rs);
            s.push(',');
            push_fx1(s, y);
            s.push(' ');
            push_fx1(s, rx + 22.0 * rs);
            s.push(',');
            push_fx1(s, y + 16.0 * rs);
            s.push(' ');
            push_fx1(s, rx + 22.0 * rs);
            s.push(',');
            push_fx1(s, y + 20.0 * rs);
            s.push_str("\" fill=\"");
            push_hex(s, COLOR_RIBBON_BLACK);
            s.push_str("\"/>\n");
        }
        let max_w = w - sc.pad_x * 2.0 + 2.0;
        let mut cy = y + sc.pad_top;
        for name in name_lines(&graph.gedcom, p.person, p.duplicate) {
            let t = trunc(fonts, name, max_w, sc.name_px, Style::Bold);
            push_text_el(
                s,
                x + w / 2.0,
                cy + sc.name_px * 0.8,
                sc.name_px,
                Style::Bold,
                COLOR_TEXT,
                &t,
            );
            cy += sc.lh_name;
        }
        let titles = titles_of(&graph.gedcom, p.person);
        if !titles.is_empty() {
            for line in titles.split('\n') {
                let t = trunc(fonts, line.to_string(), max_w, sc.title_px, Style::Italic);
                push_text_el(
                    s,
                    x + w / 2.0,
                    cy + sc.title_px * 0.8,
                    sc.title_px,
                    Style::Italic,
                    COLOR_TEXT,
                    &t,
                );
                cy += sc.lh_title;
            }
        }
        for line in date_lines(&graph.gedcom, p.person) {
            let t = trunc(fonts, line, max_w, sc.date_px, Style::Regular);
            push_text_el(
                s,
                x + w / 2.0,
                cy + sc.date_px * 0.8,
                sc.date_px,
                Style::Regular,
                COLOR_TEXT_VEILED,
                &t,
            );
            cy += sc.lh_date;
        }
    }
}
