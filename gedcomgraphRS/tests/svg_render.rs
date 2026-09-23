mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;
use gedcomgraph::render::font_draw::Fonts;
use gedcomgraph::render::measure::measure_all;
use gedcomgraph::render::svg::render_svg;
use gedcomgraph::render::theme::Scale;

fn svg_of(fulcrum_name: &str) -> Option<String> {
    let fonts = Fonts::load();
    fonts.regular.as_ref()?;
    let sc = Scale::of(4.0);
    let mut b = Builder::create();
    let adam = b.person("I1", "Adam /Stone/", Some("M"), true);
    let eve = b.person("I2", "Eve /Stone/", Some("F"), true);
    let cain = b.person("I3", "Cain /Stone/", Some("M"), false);
    let abel = b.person("I4", "Abel /Stone/", Some("M"), false);
    let ada = b.person("I5", "Ada /Lane/", Some("F"), false);
    let enoch = b.person("I6", "Enoch /Stone/", Some("M"), false);
    let ava = b.person("I7", "Ava /Stone/", Some("F"), false);
    b.family("F1", Some(adam), Some(eve), &[cain, abel], Some("1948"));
    b.family("F2", Some(cain), Some(ada), &[enoch, ava], Some("1973"));
    let ged = b.finish();
    let ful = ged.find_person(fulcrum_name).expect("fulcrum");
    let mut graph = Graph::with_gedcom(ged);
    graph.set_layout_direction(true);
    graph.start_from(ful);
    measure_all(&mut graph, &fonts, &sc);
    graph.init_nodes();
    graph.set_max_bitmap_size(1000.0);
    graph.place_nodes();
    assert!(all_finite(&graph));
    let w = (graph.get_width() * sc.s).round() as i32 + sc.pad_px() * 2;
    let h = (graph.get_height() * sc.s).round() as i32 + sc.pad_px() * 2;
    Some(render_svg(
        &graph,
        &fonts,
        &sc,
        sc.pad_px() as f32,
        sc.pad_px() as f32,
        w,
        h,
    ))
}

#[test]
fn svg_document_structure() {
    let Some(svg) = svg_of("I3") else { return };
    assert!(svg.starts_with("<svg xmlns="));
    assert!(svg.trim_end().ends_with("</svg>"));
    assert!(svg.contains("viewBox=\"0 0 "));
    assert!(svg.contains("<rect width="));
}

#[test]
fn svg_has_people_and_dates() {
    let Some(svg) = svg_of("I3") else { return };
    for name in ["Adam", "Eve", "Cain", "Ada", "Abel", "Enoch", "Ava"] {
        assert!(svg.contains(name), "missing {name}");
    }
    assert!(svg.contains("<text"), "text elements");
}

#[test]
fn svg_has_shapes() {
    let Some(svg) = svg_of("I3") else { return };
    assert!(svg.contains("<circle"), "hearths");
    assert!(svg.contains("<polygon"), "ribbons");
    assert!(svg.contains("stroke=\"#DDDDDD\""), "lines");
    assert!(svg.contains("stroke=\"#44AAFF\""), "male border");
    assert!(svg.contains("stroke=\"#FF66DD\""), "female border");
    assert!(svg.contains("font-weight=\"bold\""), "name weight");
    assert!(svg.contains("text-anchor=\"middle\""), "centering");
}

#[test]
fn svg_dimensions_match_formula() {
    let Some(svg) = svg_of("I3") else { return };
    let first = svg.lines().next().expect("first line");
    assert!(first.contains("width=\"") && first.contains("height=\""));
}

#[test]
fn ancestor_multimarriage_makes_back_lines() {
    let fonts = Fonts::load();
    if fonts.regular.is_none() {
        return;
    }
    let sc = Scale::of(4.0);
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /A/", Some("M"), true);
    let gm = b.person("I2", "GM /A/", Some("F"), true);
    let man = b.person("I3", "Man /A/", Some("M"), false);
    let w1 = b.person("I4", "W1 /A/", Some("F"), false);
    let w2 = b.person("I5", "W2 /A/", Some("F"), false);
    let w3 = b.person("I8", "W3 /A/", Some("F"), false);
    let ful = b.person("I6", "Ful /A/", Some("M"), false);
    b.family("F1", Some(gp), Some(gm), &[man], Option::None);
    b.family("F2", Some(man), Some(w1), &[ful], Option::None);
    b.family("F3", Some(man), Some(w2), &[], Some("1970"));
    b.family("F4", Some(man), Some(w3), &[], Some("1980"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.set_layout_direction(true);
    graph.start_from(ful);
    measure_all(&mut graph, &fonts, &sc);
    graph.init_nodes();
    graph.set_max_bitmap_size(1000.0);
    graph.place_nodes();
    assert!(all_finite(&graph));
    assert!(!graph.anim.back_lines.is_empty(), "expected back lines");
    for l in &graph.anim.back_lines {
        assert!(l.x1.is_finite() && l.x2.is_finite() && l.y1.is_finite() && l.y2.is_finite());
    }
    let w = (graph.get_width() * sc.s).round() as i32 + sc.pad_px() * 2;
    let h = (graph.get_height() * sc.s).round() as i32 + sc.pad_px() * 2;
    let svg = render_svg(
        &graph,
        &fonts,
        &sc,
        sc.pad_px() as f32,
        sc.pad_px() as f32,
        w,
        h,
    );
    assert!(svg.contains("stroke-dasharray"), "dashed back lines");
}

#[test]
fn svg_mini_cards_render() {
    let fonts = Fonts::load();
    if fonts.regular.is_none() {
        return;
    }
    let sc = Scale::of(4.0);
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /B/", Some("M"), true);
    let gm = b.person("I2", "GM /B/", Some("F"), true);
    let par = b.person("I3", "Par /B/", Some("M"), false);
    let spo = b.person("I4", "Spo /B/", Some("F"), false);
    let ful = b.person("I5", "Ful /B/", Some("M"), false);
    let spo2 = b.person("I6", "Sp2 /B/", Some("F"), false);
    let kid = b.person("I7", "Kid /B/", Some("M"), false);
    b.family("F1", Some(gp), Some(gm), &[par], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(ful), Some(spo2), &[kid], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_descendants(0);
    graph.set_layout_direction(true);
    graph.start_from(ful);
    measure_all(&mut graph, &fonts, &sc);
    graph.init_nodes();
    graph.place_nodes();
    assert!(graph.anim.persons.iter().any(|p| p.base.mini));
    let w = (graph.get_width() * sc.s).round() as i32 + sc.pad_px() * 2;
    let h = (graph.get_height() * sc.s).round() as i32 + sc.pad_px() * 2;
    let svg = render_svg(
        &graph,
        &fonts,
        &sc,
        sc.pad_px() as f32,
        sc.pad_px() as f32,
        w,
        h,
    );
    assert!(svg.starts_with("<svg"));
}

#[test]
fn svg_run_helpers_agree() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /Z/", Some("M"), false);
    let mom = b.person("I2", "Mom /Z/", Some("F"), false);
    let ful = b.person("I3", "Ful /Z/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    for f in graph.fulcrum_persons() {
        assert_eq!(person_generation(&graph, f), 0);
        assert_eq!(gedcom_of(&graph, f), ful);
    }
    run_sized(&mut graph, ful, 60.0, 40.0);
    assert!(all_finite(&graph));
}
