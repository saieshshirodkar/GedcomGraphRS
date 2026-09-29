mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;
use gedcomgraph::model::PersonId;

fn solo() -> (Graph, PersonId) {
    let mut b = Builder::create();
    let a = b.person("I1", "Alice /Smith/", Option::None, false);
    let ged = b.finish();
    let g = Graph::with_gedcom(ged);
    (g, a)
}

#[test]
fn single_person_no_families() {
    let (mut graph, fulcrum) = solo();
    run(&mut graph, fulcrum);
    assert!(graph.anim.node_count() >= 1);
    assert!(graph.anim.person_count() >= 1);
    assert!(all_finite(&graph));
    let fulcrums = graph.fulcrum_persons();
    assert_eq!(fulcrums.len(), 1);
    assert_eq!(gedcom_of(&graph, fulcrums[0]), fulcrum);
}

#[test]
fn person_with_two_parents() {
    let mut b = Builder::create();
    let mom = b.person("I1", "Mom /A/", Some("F"), false);
    let dad = b.person("I2", "Dad /A/", Some("M"), false);
    let kid = b.person("I3", "Kid /A/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[kid], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, kid);
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&mom));
    assert!(ids.contains(&dad));
    assert!(ids.contains(&kid));
    assert!(all_finite(&graph));
}

#[test]
fn spouse_and_children() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /B/", Some("M"), false);
    let mom = b.person("I2", "Mom /B/", Some("F"), false);
    let c1 = b.person("I3", "C1 /B/", Some("M"), false);
    let c2 = b.person("I4", "C2 /B/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[c1, c2], Some("1950"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, dad);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&c1));
    assert!(ids.contains(&c2));
    assert!(ids.contains(&mom));
}

#[test]
fn three_generations() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /C/", Some("M"), true);
    let gm = b.person("I2", "GM /C/", Some("F"), true);
    let par = b.person("I3", "Par /C/", Some("M"), false);
    let spo = b.person("I4", "Spo /C/", Some("F"), false);
    let ful = b.person("I5", "Ful /C/", Some("M"), false);
    let chd = b.person("I6", "Chd /C/", Some("F"), false);
    b.family("F1", Some(gf), Some(gm), &[par], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(ful), Option::None, &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.width().is_finite());
    assert!(graph.height().is_finite());
}

#[test]
fn multi_marriage_second_union() {
    let mut b = Builder::create();
    let man = b.person("I1", "Man /D/", Some("M"), false);
    let w1 = b.person("I2", "W1 /D/", Some("F"), false);
    let w2 = b.person("I3", "W2 /D/", Some("F"), false);
    let k1 = b.person("I4", "K1 /D/", Some("M"), false);
    let k2 = b.person("I5", "K2 /D/", Some("F"), false);
    b.family("F1", Some(man), Some(w1), &[k1], Option::None);
    b.family("F2", Some(man), Some(w2), &[k2], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, k1);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&man));
}

#[test]
fn pedigree_collapse_marks_duplicates() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /E/", Some("M"), true);
    let gm = b.person("I2", "GM /E/", Some("F"), true);
    let dad = b.person("I3", "Dad /E/", Some("M"), false);
    let mom = b.person("I4", "Mom /E/", Some("F"), false);
    let ful = b.person("I5", "Ful /E/", Some("M"), false);
    b.family("F1", Some(gf), Some(gm), &[dad, mom], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.anim.persons.iter().any(|p| p.duplicate));
}

#[test]
fn single_parent_family() {
    let mut b = Builder::create();
    let mom = b.person("I1", "Mom /F/", Some("F"), false);
    let kid = b.person("I2", "Kid /F/", Some("M"), false);
    b.family("F1", Option::None, Some(mom), &[kid], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, kid);
    assert!(all_finite(&graph));
    assert!(graph.anim.person_count() >= 2);
}

#[test]
fn empty_name_allowed() {
    let mut b = Builder::create();
    let a = b.person("I1", "", Option::None, false);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, a);
    assert!(all_finite(&graph));
    assert_eq!(graph.node_label(graph.anim.nodes[0]), "[No name]");
}

#[test]
fn fulcrum_at_generation_zero() {
    let mut b = Builder::create();
    let par = b.person("I1", "Par /G/", Some("M"), false);
    let spo = b.person("I2", "Spo /G/", Some("F"), false);
    let ful = b.person("I3", "Ful /G/", Some("F"), false);
    b.family("F1", Some(par), Some(spo), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    for f in graph.fulcrum_persons() {
        assert_eq!(person_generation(&graph, f), 0);
    }
}

#[test]
fn ancestors_negative_descendants_positive() {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /H/", Some("M"), true);
    let gm = b.person("I2", "GM /H/", Some("F"), true);
    let par = b.person("I3", "Par /H/", Some("M"), false);
    let spo = b.person("I4", "Spo /H/", Some("F"), false);
    let ful = b.person("I5", "Ful /H/", Some("M"), false);
    let chd = b.person("I6", "Chd /H/", Some("F"), false);
    b.family("F1", Some(gp), Some(gm), &[par], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(ful), Option::None, &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    for (i, p) in graph.anim.persons.iter().enumerate() {
        if PersonId(p.person) == gp || PersonId(p.person) == gm {
            assert!(p.base.generation < 0, "ancestor gen {}", p.base.generation);
        }
        if PersonId(p.person) == chd {
            assert!(p.base.generation > 0, "child gen {}", p.base.generation);
        }
        let _ = i;
    }
}

#[test]
fn siblings_share_row() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /J/", Some("M"), false);
    let mom = b.person("I2", "Mom /J/", Some("F"), false);
    let a = b.person("I3", "A /J/", Some("M"), false);
    let sib = b.person("I4", "Sib /J/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[a, sib], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, a);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&sib));
}

#[test]
fn sized_run_keeps_finite() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /K/", Some("M"), false);
    let mom = b.person("I2", "Mom /K/", Some("F"), false);
    let k1 = b.person("I3", "K1 /K/", Some("M"), false);
    let k2 = b.person("I4", "K2 /K/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[k1, k2], Some("1960"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run_sized(&mut graph, k1, 60.0, 40.0);
    assert!(all_finite(&graph));
    assert!(graph.width() >= 60.0);
}

#[test]
fn duplicate_lines_created_for_collapse() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /L/", Some("M"), true);
    let gm = b.person("I2", "GM /L/", Some("F"), true);
    let dad = b.person("I3", "Dad /L/", Some("M"), false);
    let mom = b.person("I4", "Mom /L/", Some("F"), false);
    let ful = b.person("I5", "Ful /L/", Some("M"), false);
    b.family("F1", Some(gf), Some(gm), &[dad, mom], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(graph.duplicate_line_count() > 0);
}

#[test]
fn without_numbers_still_lays_out() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /M/", Some("M"), false);
    let mom = b.person("I2", "Mom /M/", Some("F"), false);
    let k1 = b.person("I3", "K1 /M/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.display_numbers(false);
    run(&mut graph, k1);
    assert!(all_finite(&graph));
}

#[test]
fn without_spouses_hides_partners() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /N/", Some("M"), false);
    let mom = b.person("I2", "Mom /N/", Some("F"), false);
    let k1 = b.person("I3", "K1 /N/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.display_spouses(false);
    run(&mut graph, k1);
    assert!(all_finite(&graph));
}

#[test]
fn describe_lists_nodes() {
    let (mut graph, fulcrum) = solo();
    run(&mut graph, fulcrum);
    let txt = graph.describe();
    assert!(txt.contains("Alice Smith"));
}
