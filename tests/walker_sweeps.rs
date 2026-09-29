mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::lines::LineKind;
use gedcomgraph::model::PersonId;
use gedcomgraph::{Card, Graph, Match};

fn uncles() -> (Graph, PersonId, PersonId, PersonId) {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /A/", Some("M"), true);
    let gm = b.person("I2", "GM /A/", Some("F"), true);
    let par = b.person("I3", "Par /A/", Some("M"), false);
    let unc = b.person("I4", "Unc /A/", Some("M"), false);
    let aunt = b.person("I5", "Aunt /A/", Some("F"), false);
    let spo = b.person("I6", "Spo /A/", Some("F"), false);
    let ful = b.person("I7", "Ful /A/", Some("M"), false);
    let cou = b.person("I8", "Cou /A/", Some("M"), false);
    b.family("F1", Some(gp), Some(gm), &[par, unc], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(unc), Some(aunt), &[cou], Option::None);
    let ged = b.finish();
    (Graph::with_gedcom(ged), ful, unc, cou)
}

fn spouses3() -> (Graph, PersonId) {
    let mut b = Builder::create();
    let man = b.person("I1", "Man /B/", Some("M"), false);
    let w1 = b.person("I2", "W1 /B/", Some("F"), false);
    let w2 = b.person("I3", "W2 /B/", Some("F"), false);
    let w3 = b.person("I4", "W3 /B/", Some("F"), false);
    b.family("F1", Some(man), Some(w1), &[], Option::None);
    b.family("F2", Some(man), Some(w2), &[], Option::None);
    b.family("F3", Some(man), Some(w3), &[], Option::None);
    let ged = b.finish();
    (Graph::with_gedcom(ged), man)
}

fn three_gen() -> (Graph, PersonId) {
    let mut b = Builder::create();
    let par = b.person("I1", "Par /C/", Some("M"), false);
    let spo = b.person("I2", "Spo /C/", Some("F"), false);
    let ful = b.person("I3", "Ful /C/", Some("M"), false);
    let ptn = b.person("I4", "Ptn /C/", Some("F"), false);
    let chd = b.person("I5", "Chd /C/", Some("M"), false);
    b.family("F1", Some(par), Some(spo), &[ful], Option::None);
    b.family("F2", Some(ful), Some(ptn), &[chd], Option::None);
    let ged = b.finish();
    (Graph::with_gedcom(ged), ful)
}

fn ids(graph: &Graph) -> Vec<PersonId> {
    graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect()
}

#[test]
fn great_uncles_zero_keeps_cousins() {
    let (mut graph, ful, unc, _) = uncles();
    graph.max_great_uncles(0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(ids(&graph).contains(&unc));
}

#[test]
fn no_uncles_when_both_zero() {
    let (mut graph, ful, unc, cou) = uncles();
    graph.max_great_uncles(0).max_uncles_cousins(0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(!ids(&graph).contains(&unc));
    assert!(!ids(&graph).contains(&cou));
}

#[test]
fn no_siblings_at_zero() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /D/", Some("M"), false);
    let mom = b.person("I2", "Mom /D/", Some("F"), false);
    let ful = b.person("I3", "Ful /D/", Some("M"), false);
    let sib = b.person("I4", "Sib /D/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[ful, sib], Option::None);
    let mut graph = Graph::with_gedcom(b.finish());
    graph.max_siblings_nephews(0);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(!ids(&graph).contains(&sib));
}

#[test]
fn ancestors_zero_yields_mini() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /E/", Some("M"), false);
    let mom = b.person("I2", "Mom /E/", Some("F"), false);
    let ful = b.person("I3", "Ful /E/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[ful], Option::None);
    let mut graph = Graph::with_gedcom(b.finish());
    graph.max_ancestors(0).display_numbers(true);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let minis: Vec<_> = graph
        .anim
        .persons
        .iter()
        .filter(|p| p.kind == Card::Ancestry)
        .collect();
    assert!(!minis.is_empty());
    for m in minis {
        assert!(m.base.mini);
        assert!(m.amount > 0);
    }
}

#[test]
fn half_sibling_flagged() {
    let mut bb = Builder::create();
    let dad = bb.person("I1", "Dad /F/", Some("M"), false);
    let m1 = bb.person("I2", "M1 /F/", Some("F"), false);
    let m2 = bb.person("I3", "M2 /F/", Some("F"), false);
    let ful = bb.person("I4", "Ful /F/", Some("M"), false);
    let half = bb.person("I5", "Half /F/", Some("F"), false);
    bb.family("F1", Some(dad), Some(m1), &[ful], Option::None);
    bb.family("F2", Some(dad), Some(m2), &[half], Option::None);
    let mut graph = Graph::with_gedcom(bb.finish());
    graph.display_spouses(false);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.anim.persons.iter().any(|p| p.half_sibling));
}

#[test]
fn three_spouse_next_and_back_kinds() {
    let (mut graph, man) = spouses3();
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, man);
    assert!(all_finite(&graph));
    assert!(
        graph
            .anim
            .lines
            .iter()
            .any(|l| matches!(l.kind, LineKind::Next { .. })),
        "Next line present"
    );
    assert!(
        graph
            .anim
            .back_lines
            .iter()
            .any(|l| matches!(l.kind, LineKind::Back { .. })),
        "Back line present"
    );
    assert!(!graph.anim.back_line_groups.is_empty());
}

#[test]
fn dup_indices_in_range() {
    let mut b = Builder::create();
    let gf = b.person("I1", "GF /G/", Some("M"), true);
    let gm = b.person("I2", "GM /G/", Some("F"), true);
    let dad = b.person("I3", "Dad /G/", Some("M"), false);
    let mom = b.person("I4", "Mom /G/", Some("F"), false);
    let ful = b.person("I5", "Ful /G/", Some("M"), false);
    b.family("F1", Some(gf), Some(gm), &[dad, mom], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    let mut graph = Graph::with_gedcom(b.finish());
    run(&mut graph, ful);
    assert!(!graph.anim.duplicate_lines.is_empty());
    for d in &graph.anim.duplicate_lines {
        assert!((d.first as usize) < graph.anim.person_count());
        assert!((d.second as usize) < graph.anim.person_count());
        assert!(d.x3.is_finite() && d.y3.is_finite());
    }
}

#[test]
fn init_only_solo() {
    let mut b = Builder::create();
    let a = b.person("I1", "Solo /H/", Some("M"), false);
    let mut graph = Graph::with_gedcom(b.finish());
    graph.start_from(a.0);
    graph.init_nodes();
    assert!(graph.anim.node_count() >= 1);
    assert!(graph.anim.person_count() >= 1);
    assert_eq!(person_generation(&graph, 0), 0);
    assert_eq!(gedcom_of(&graph, 0), a);
}

#[test]
fn bitmap_zero_biggest_zero() {
    let mut b = Builder::create();
    let a = b.person("I1", "Solo /I/", Some("M"), false);
    let mut graph = Graph::with_gedcom(b.finish());
    assert!(graph.need_max_bitmap_size());
    run(&mut graph, a);
    assert_eq!(graph.biggest_path_size(), 0.0);
}

#[test]
fn mirror_preserves_layout() {
    let (mut g1, ful) = three_gen();
    run_sized(&mut g1, ful, 60.0, 40.0);
    let (mut g2, ful2) = three_gen();
    g2.set_layout_direction(true);
    run_sized(&mut g2, ful2, 60.0, 40.0);
    assert_eq!(g1.width(), g2.width());
    assert_eq!(g1.height(), g2.height());
    assert_eq!(g1.anim.person_count(), g2.anim.person_count());
    for i in 0..g1.anim.person_count() {
        let a = &g1.anim.persons[i];
        let b = &g2.anim.persons[i];
        assert!((a.base.x + b.base.x + a.base.w - g1.width()).abs() < 0.05);
        assert_eq!(a.base.y, b.base.y);
    }
}

#[test]
fn generations_contiguous() {
    let (mut graph, ful) = three_gen();
    run(&mut graph, ful);
    let gens: Vec<i32> = graph
        .anim
        .persons
        .iter()
        .map(|p| p.base.generation)
        .collect();
    assert!(!gens.is_empty());
    let (lo, hi) = (
        gens.iter().min().copied().unwrap(),
        gens.iter().max().copied().unwrap(),
    );
    for g in lo..=hi {
        assert!(gens.contains(&g), "generation {g} present");
    }
    assert!(gens.contains(&0));
}

#[test]
fn multimarr_has_nonmain_stamp() {
    let (mut graph, man) = spouses3();
    run(&mut graph, man);
    assert!(
        graph
            .anim
            .families
            .iter()
            .any(|f| f.base.stamp != Match::Main),
        "a multi-marriage family is stamped"
    );
}

#[test]
fn acquired_have_no_origin() {
    let (mut graph, ful) = three_gen();
    graph.display_numbers(true);
    run(&mut graph, ful);
    for p in &graph.anim.persons {
        if p.acquired {
            assert_eq!(p.origin, Option::None);
        }
    }
}

#[test]
fn curves_point_downward() {
    let (mut graph, ful) = three_gen();
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, ful);
    let mut found = false;
    for l in &graph.anim.lines {
        if let LineKind::Curve { .. } = l.kind {
            found = true;
            assert!(l.y1 < l.y2, "curve descends {} < {}", l.y1, l.y2);
        }
    }
    assert!(found, "curves exist");
}
