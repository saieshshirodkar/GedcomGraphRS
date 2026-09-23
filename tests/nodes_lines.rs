mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;

#[test]
fn multi_marriage_creates_next_and_back_lines() {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /A/", Some("M"), true);
    let gm = b.person("I2", "GM /A/", Some("F"), true);
    let man = b.person("I3", "Man /A/", Some("M"), false);
    let w1 = b.person("I4", "W1 /A/", Some("F"), false);
    let w2 = b.person("I5", "W2 /A/", Some("F"), false);
    let ful = b.person("I6", "Ful /A/", Some("M"), false);
    b.family("F1", Some(gp), Some(gm), &[man], Option::None);
    b.family("F2", Some(man), Some(w1), &[ful], Option::None);
    b.family("F3", Some(man), Some(w2), &[], Some("1970"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.line_count() > 0);
}

#[test]
fn lines_have_finite_coords() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /B/", Some("M"), false);
    let mom = b.person("I2", "Mom /B/", Some("F"), false);
    let k1 = b.person("I3", "K1 /B/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1], Some("1955"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.set_max_bitmap_size(2000.0);
    run_sized(&mut graph, k1, 60.0, 40.0);
    for l in &graph.anim.lines {
        assert!(l.x1.is_finite() && l.y1.is_finite() && l.x2.is_finite() && l.y2.is_finite());
    }
    for l in &graph.anim.back_lines {
        assert!(l.x1.is_finite() && l.y1.is_finite() && l.x2.is_finite() && l.y2.is_finite());
    }
    assert!(!graph.anim.line_groups.is_empty());
}

#[test]
fn duplicate_lines_have_third_point() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /C/", Some("M"), true);
    let gm = b.person("I2", "GM /C/", Some("F"), true);
    let dad = b.person("I3", "Dad /C/", Some("M"), false);
    let mom = b.person("I4", "Mom /C/", Some("F"), false);
    let ful = b.person("I5", "Ful /C/", Some("M"), false);
    b.family("F1", Some(gf), Some(gm), &[dad, mom], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(!graph.anim.duplicate_lines.is_empty());
    for d in &graph.anim.duplicate_lines {
        assert!(d.x3.is_finite() && d.y3.is_finite());
    }
}

#[test]
fn no_duplicate_lines_when_disabled() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /D/", Some("M"), true);
    let gm = b.person("I2", "GM /D/", Some("F"), true);
    let dad = b.person("I3", "Dad /D/", Some("M"), false);
    let mom = b.person("I4", "Mom /D/", Some("F"), false);
    let ful = b.person("I5", "Ful /D/", Some("M"), false);
    b.family("F1", Some(gf), Some(gm), &[dad, mom], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.display_duplicate_lines(false);
    run(&mut graph, ful);
    assert!(graph.anim.duplicate_lines.is_empty());
    assert!(graph.anim.persons.iter().any(|p| p.duplicate));
}

#[test]
fn bonds_cover_all_couples() {
    let mut b = Builder::create();
    let a = b.person("I1", "A /E/", Some("M"), false);
    let c = b.person("I2", "C /E/", Some("F"), false);
    let k = b.person("I3", "K /E/", Some("M"), false);
    b.family("F1", Some(a), Some(c), &[k], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, k);
    assert_eq!(
        graph.anim.bond_views().len(),
        graph
            .anim
            .families
            .iter()
            .filter(|f| f.bond.is_some())
            .count()
    );
}

#[test]
fn single_parent_has_no_bond_line() {
    let mut b = Builder::create();
    let solo = b.person("I1", "Solo /F/", Some("M"), false);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, solo);
    assert!(graph.anim.bond_views().is_empty());
    assert!(graph.line_count() == 0);
    assert_eq!(person_generation(&graph, 0), 0);
    assert_eq!(gedcom_of(&graph, 0), solo);
}
