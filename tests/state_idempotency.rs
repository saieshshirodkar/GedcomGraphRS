mod common;

use common::Builder;
use gedcomgraph::render::font_draw::Fonts;
use gedcomgraph::render::svg::render_svg;
use gedcomgraph::render::theme::Scale;
use gedcomgraph::{Graph, parse_gedcom};

fn tree() -> gedcomgraph::GedcomData {
    let mut b = Builder::create();
    let gf = b.person("GF", "Gran /Old/", Some("M"), false);
    let gm = b.person("GM", "Granm /Old/", Some("F"), false);
    let f = b.person("F", "Father /Mid/", Some("M"), false);
    let m = b.person("M", "Mother /Mid/", Some("F"), false);
    let k1 = b.person("K1", "Kid /One/", Some("M"), false);
    let k2 = b.person("K2", "Kid /Two/", Some("F"), false);
    let s1 = b.person("S1", "Spouse /One/", Some("F"), false);
    let s2 = b.person("S2", "Spouse /Two/", Some("M"), false);
    b.family("FF1", Some(gf), Some(gm), &[f], None);
    b.family("FF2", Some(f), Some(m), &[k1, k2], Some("1 JAN 1980"));
    b.family("FF3", Some(k1), Some(s1), &[], None);
    b.family("FF4", Some(k1), Some(s2), &[], None);
    b.finish()
}

fn configured(g: &mut Graph) {
    g.max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10)
        .display_spouses(true)
        .display_numbers(true)
        .display_duplicate_lines(true)
        .set_layout_direction(true);
}

fn svg_of(g: &Graph, fonts: &Fonts, sc: &Scale) -> String {
    render_svg(
        g,
        fonts,
        sc,
        sc.pad_px() as f32,
        sc.pad_px() as f32,
        4000,
        4000,
    )
}

#[test]
fn init_nodes_twice_is_idempotent() {
    let ged = tree();
    let ful = ged.find_person("K1").expect("fulcrum").0;
    let fonts = Fonts::load();
    let sc = Scale::of(4.0);

    let mut once = Graph::with_gedcom(ged.clone());
    configured(&mut once);
    once.start_from(ful);
    once.init_nodes();
    once.place_nodes();
    let a = svg_of(&once, &fonts, &sc);
    let bonds_a = once.anim.bond_owners.len();
    let unions_a = once.anim.unions.len();

    let mut twice = Graph::with_gedcom(ged);
    configured(&mut twice);
    twice.start_from(ful);
    twice.init_nodes();
    twice.init_nodes();
    twice.place_nodes();
    let b = svg_of(&twice, &fonts, &sc);

    assert_eq!(twice.anim.bond_owners.len(), bonds_a);
    assert_eq!(twice.anim.unions.len(), unions_a);
    assert_eq!(
        (twice.width(), twice.height()),
        (once.width(), once.height())
    );
    assert_eq!(a, b, "second init_nodes must not change the render");
}

#[test]
fn place_nodes_twice_is_idempotent() {
    let ged = tree();
    let ful = ged.find_person("K1").expect("fulcrum").0;
    let fonts = Fonts::load();
    let sc = Scale::of(4.0);

    let mut g = Graph::with_gedcom(ged);
    configured(&mut g);
    g.start_from(ful);
    g.init_nodes();
    g.place_nodes();
    let a = svg_of(&g, &fonts, &sc);
    g.place_nodes();
    assert_eq!(a, svg_of(&g, &fonts, &sc));
}

#[test]
fn repeated_walks_do_not_grow_state() {
    let ged = tree();
    let ful = ged.find_person("K1").expect("fulcrum").0;
    let mut g = Graph::with_gedcom(ged);
    configured(&mut g);

    g.start_from(ful);
    g.init_nodes();
    g.place_nodes();
    let unions = g.anim.unions.len();
    let groups = g.anim.groups.len();
    let bonds = g.anim.bond_owners.len();

    for _ in 0..25 {
        g.start_from(ful);
        g.init_nodes();
        g.place_nodes();
    }
    assert_eq!(g.anim.unions.len(), unions);
    assert_eq!(g.anim.groups.len(), groups);
    assert_eq!(g.anim.bond_owners.len(), bonds);
}

#[test]
fn walking_a_different_fulcrum_is_clean() {
    let ged = tree();
    let other = ged.find_person("K2").expect("fulcrum").0;
    let ful = ged.find_person("K1").expect("fulcrum").0;
    let mut g = Graph::with_gedcom(ged);
    configured(&mut g);

    g.start_from(ful);
    g.init_nodes();
    g.place_nodes();
    let unions = g.anim.unions.len();

    g.start_from(other);
    g.init_nodes();
    g.place_nodes();
    assert_eq!(g.anim.unions.len(), unions);
}

#[test]
fn repeated_walks_on_real_file_are_stable() {
    let path = format!("{}/example.ged", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).expect("example.ged");
    let ged = parse_gedcom(&text);
    let ful = ged.find_person("I3").expect("fulcrum").0;
    let fonts = Fonts::load();
    let sc = Scale::of(4.0);

    let mut g = Graph::with_gedcom(ged);
    configured(&mut g);
    g.start_from(ful);
    g.init_nodes();
    g.place_nodes();
    let a = svg_of(&g, &fonts, &sc);
    let unions = g.anim.unions.len();
    let bonds = g.anim.bond_owners.len();

    for _ in 0..10 {
        g.start_from(ful);
        g.init_nodes();
        g.place_nodes();
    }
    assert_eq!(g.anim.unions.len(), unions);
    assert_eq!(g.anim.bond_owners.len(), bonds);
    assert_eq!(a, svg_of(&g, &fonts, &sc));
}
