mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;
use gedcomgraph::model::PersonId;

fn tree() -> (Graph, PersonId, PersonId, PersonId, PersonId) {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /A/", Some("M"), false);
    let mom = b.person("I2", "Mom /A/", Some("F"), false);
    let ful = b.person("I3", "Ful /A/", Some("M"), false);
    let sib = b.person("I4", "Sib /A/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[ful, sib], Option::None);
    let ged = b.finish();
    (Graph::with_gedcom(ged), dad, mom, ful, sib)
}

#[test]
fn config_chain_sets_values() {
    let (mut graph, _, _, ful, _) = tree();
    graph
        .max_ancestors(5)
        .max_descendants(4)
        .max_great_uncles(3)
        .max_siblings_nephews(2)
        .max_uncles_cousins(1)
        .display_spouses(true)
        .display_numbers(true)
        .display_duplicate_lines(false);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert_eq!(graph.ancestor_generations, 5);
    assert_eq!(graph.descendant_generations, 4);
    assert_eq!(graph.great_uncles_generations, 3);
    assert_eq!(graph.sibling_nephew_generations, 2);
    assert_eq!(graph.uncle_cousin_generations, 1);
}

#[test]
fn numbers_toggle_changes_spacing() {
    let (mut graph, _, _, ful, _) = tree();
    graph.display_numbers(true);
    let full = graph.vertical_calc();
    graph.display_numbers(false);
    assert!(graph.vertical_calc() < full);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn bitmap_size_gating() {
    let (mut graph, _, _, ful, _) = tree();
    assert!(graph.need_max_bitmap_size());
    graph.set_max_bitmap_size(2000.0);
    assert!(!graph.need_max_bitmap_size());
    assert_eq!(graph.max_bitmap_size(), 2000.0);
    run(&mut graph, ful);
    assert!(graph.biggest_path_size() >= 0.0);
}

#[test]
fn left_to_right_mirrors() {
    let (mut g1, _, _, ful, _) = tree();
    run_sized(&mut g1, ful, 60.0, 40.0);
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /A/", Some("M"), false);
    let mom = b.person("I2", "Mom /A/", Some("F"), false);
    let ful2 = b.person("I3", "Ful /A/", Some("M"), false);
    let sib = b.person("I4", "Sib /A/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[ful2, sib], Option::None);
    let mut g2 = Graph::with_gedcom(b.finish());
    g2.set_layout_direction(true);
    run_sized(&mut g2, ful2, 60.0, 40.0);
    assert_eq!(g1.width(), g2.width());
    assert_eq!(g1.height(), g2.height());
}

#[test]
fn which_family_clamped() {
    let mut b = Builder::create();
    let p1 = b.person("I1", "P1 /B/", Some("M"), false);
    let p2 = b.person("I2", "P2 /B/", Some("F"), false);
    let ful = b.person("I3", "Ful /B/", Some("M"), false);
    b.family("F1", Some(p1), Some(p2), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.show_family(99);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn zero_ancestor_generations() {
    let (mut graph, _, _, ful, _) = tree();
    graph.max_ancestors(0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert_eq!(graph.max_above, 0);
}

#[test]
fn zero_descendant_generations() {
    let mut b = Builder::create();
    let ful = b.person("I1", "Ful /C/", Some("M"), false);
    let spo = b.person("I2", "Spo /C/", Some("F"), false);
    let chd = b.person("I3", "Chd /C/", Some("M"), false);
    b.family("F1", Some(ful), Some(spo), &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_descendants(0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn spouses_lookup_order() {
    let (graph, dad, mom, _, _) = tree();
    let spouses = graph.spouses(0, Option::None);
    assert_eq!(spouses, vec![dad, mom]);
    assert_eq!(graph.spouses(0, Some(dad.0)), vec![mom]);
}

#[test]
fn siblings_detection() {
    let (mut graph, _, _, ful, sib) = tree();
    run(&mut graph, ful);
    let ful_node = graph
        .anim
        .persons
        .iter()
        .position(|p| PersonId(p.person) == ful)
        .map(|i| i as u32);
    let sib_node = graph
        .anim
        .persons
        .iter()
        .position(|p| PersonId(p.person) == sib)
        .map(|i| i as u32);
    assert!(ful_node.is_some());
    assert!(sib_node.is_some());
    assert!(graph.are_siblings(ful_node, sib_node));
    assert!(!graph.are_siblings(Option::None, sib_node));
    assert!(!graph.are_siblings(ful_node, Option::None));
}

#[test]
fn default_spacing_matches_numbered() {
    let g = Graph::create();
    assert_eq!(g.vertical_calc(), Graph::vertical_space_default());
}

#[test]
fn person_size_api() {
    let (mut graph, _, _, ful, _) = tree();
    graph.start_from(ful.0);
    graph.set_person_size(0, 80.0, 50.0);
    graph.set_all_person_sizes(70.0, 45.0);
    graph.init_nodes();
    graph.place_nodes();
    assert!(all_finite(&graph));
    for f in graph.fulcrum_persons() {
        assert_eq!(person_generation(&graph, f), 0);
        assert_eq!(
            graph.person_gedcom_id(f),
            graph.gedcom_person_id(gedcom_of(&graph, f).0)
        );
    }
}
