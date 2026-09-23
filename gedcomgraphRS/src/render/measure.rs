use crate::engine::graph::Graph;
use crate::render::font_draw::{Fonts, Style};
use crate::render::texts::{date_lines, name_lines, titles_of};
use crate::render::theme::Scale;

pub fn measure_all(graph: &mut Graph, fonts: &Fonts, sc: &Scale) {
    let ids: Vec<u32> = (0..graph.anim.person_count() as u32).collect();
    for pid in ids {
        let p = &graph.anim.persons[pid as usize];
        let (mini, amount, person, duplicate) = (p.base.mini, p.amount, p.person, p.duplicate);
        if mini {
            let txt = if amount > 100 {
                "100+".to_string()
            } else {
                amount.to_string()
            };
            let tw = fonts.measure(&txt, sc.mini_px, Style::Bold);
            graph.set_person_size(
                pid,
                (tw + (14.0 * sc.s).round()) / sc.s,
                ((14.0 * sc.s).round() + sc.mini_box().1) / sc.s,
            );
        } else {
            let names = name_lines(&graph.gedcom, person, duplicate);
            let mut nw = 0.0f32;
            for l in &names {
                nw = nw.max(fonts.measure(l, sc.name_px, Style::Bold));
            }
            let titles = titles_of(&graph.gedcom, person);
            let tls: Vec<&str> = if titles.is_empty() {
                Vec::new()
            } else {
                titles.split('\n').collect()
            };
            let mut tw = 0.0f32;
            for l in &tls {
                tw = tw.max(fonts.measure(l, sc.title_px, Style::Italic));
            }
            let dates = date_lines(&graph.gedcom, person);
            let mut dw = 0.0f32;
            for l in &dates {
                dw = dw.max(fonts.measure(l, sc.date_px, Style::Regular));
            }
            let w = sc.card_min_w.max(nw.max(tw).max(dw) + sc.pad_x * 2.0 + 1.0) / sc.s;
            let mut h = sc.pad_top + names.len() as f32 * sc.lh_name;
            if !tls.is_empty() {
                h += tls.len() as f32 * sc.lh_title;
            }
            if !dates.is_empty() {
                h += dates.len() as f32 * sc.lh_date;
            }
            h += sc.pad_bot;
            graph.set_person_size(pid, w, h / sc.s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::GedcomData;
    use crate::model::gedcom::{Fact, GPerson, Name};
    use crate::render::theme::Scale;

    #[test]
    fn sizes_are_positive() {
        let fonts = Fonts::load();
        if fonts.regular.is_none() {
            return;
        }
        let sc = Scale::of(4.0);
        let mut ged = GedcomData::empty();
        let a = ged.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("Adam /Stone/")],
            facts: vec![Fact::dated("BIRT", Some("1920"))],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        ged.reindex();
        let mut graph = Graph::with_gedcom(ged);
        graph.start_from(a);
        measure_all(&mut graph, &fonts, &sc);
        for p in &graph.anim.persons {
            assert!(p.base.w > 0.0);
            assert!(p.base.h > 0.0);
        }
    }
}
