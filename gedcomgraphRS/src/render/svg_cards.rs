use crate::engine::graph::Graph;
use crate::render::font_draw::{Fonts, Style};
use crate::render::svg::{alpha, border_of, hex, text_el, trunc};
use crate::render::texts::{date_lines, is_dead, name_lines, titles_of};
use crate::render::theme::{
    COLOR_BACK_ELEMENT, COLOR_PARTNER, COLOR_RIBBON_BLACK, COLOR_RIBBON_WHITE, COLOR_TEXT,
    COLOR_TEXT_VEILED, Scale,
};

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
        s.push_str(&format!(
            "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{:.1}\" fill=\"{}\"{}/>\n",
            sc.bg_corner,
            hex(bg),
            alpha(bg)
        ));
        s.push_str(&format!(
            "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{:.1}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.1}\"/>\n",
            sc.corner,
            hex(border),
            sc.border
        ));
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
