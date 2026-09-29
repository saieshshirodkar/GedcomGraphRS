mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;
use gedcomgraph::model::PersonId;

fn generations() -> (
    Graph,
    PersonId,
    PersonId,
    PersonId,
    PersonId,
    PersonId,
    PersonId,
) {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /A/", Some("M"), true);
    let gm = b.person("I2", "GM /A/", Some("F"), true);
    let par = b.person("I3", "Par /A/", Some("M"), false);
    let spo = b.person("I4", "Spo /A/", Some("F"), false);
    let ful = b.person("I5", "Ful /A/", Some("M"), false);
    let chd = b.person("I6", "Chd /A/", Some("F"), false);
    b.family("F1", Some(gp), Some(gm), &[par], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(ful), Option::None, &[chd], Option::None);
    let ged = b.finish();
    (Graph::with_gedcom(ged), gp, gm, par, ful, chd, spo)
}

#[test]
fn y_ordered_by_generation() {
    let (mut graph, gp, _, par, ful, chd, _) = generations();
    run(&mut graph, ful);
    let y_of = |id: PersonId| -> f32 {
        graph
            .anim
            .persons
            .iter()
            .find(|p| PersonId(p.person) == id)
            .map(|p| p.base.y)
            .unwrap_or(f32::NAN)
    };
    assert!(y_of(gp) <= y_of(par));
    assert!(y_of(par) <= y_of(ful));
    assert!(y_of(ful) <= y_of(chd));
}

#[test]
fn generation_gap_uses_vertical_space() {
    let (mut graph, _, _, _, ful, _, _) = generations();
    graph.display_numbers(true);
    run(&mut graph, ful);
    let mut ys: Vec<f32> = graph.anim.persons.iter().map(|p| p.base.y).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for w in ys.windows(2) {
        assert!(w[1] - w[0] >= -0.01);
    }
}

#[test]
fn sized_nodes_do_not_overlap_in_row() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /B/", Some("M"), false);
    let mom = b.person("I2", "Mom /B/", Some("F"), false);
    let k1 = b.person("I3", "K1 /B/", Some("M"), false);
    let k2 = b.person("I4", "K2 /B/", Some("F"), false);
    let k3 = b.person("I5", "K3 /B/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1, k2, k3], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.set_max_bitmap_size(2000.0);
    run_sized(&mut graph, k1, 60.0, 40.0);
    let mut kids: Vec<(f32, f32)> = graph
        .anim
        .persons
        .iter()
        .filter(|p| p.base.generation == 1)
        .map(|p| (p.base.x, p.base.w))
        .collect();
    kids.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for w in kids.windows(2) {
        assert!(w[0].0 + w[0].1 <= w[1].0 + 0.01, "overlap {w:?}");
    }
}

#[test]
fn fulcrum_family_shares_y() {
    let (mut graph, _, _, ful, _, _, _) = generations();
    run_sized(&mut graph, ful, 60.0, 40.0);
    let ful_y = graph
        .anim
        .persons
        .iter()
        .find(|p| p.is_fulcrum())
        .map(|p| p.base.y)
        .unwrap_or(f32::NAN);
    assert!(ful_y.is_finite());
    assert!(all_finite(&graph));
    for f in graph.fulcrum_persons() {
        assert_eq!(person_generation(&graph, f), 0);
        assert!(!graph.person_gedcom_id(f).is_empty());
        assert_eq!(
            gedcom_of(&graph, f),
            graph
                .gedcom
                .find_person(&graph.person_gedcom_id(f))
                .unwrap_or(PersonId(u32::MAX))
        );
    }
}

#[test]
fn width_covers_all_nodes() {
    let (mut graph, _, _, _, ful, _, _) = generations();
    run_sized(&mut graph, ful, 60.0, 40.0);
    let w = graph.width();
    for n in graph.anim.nodes.clone() {
        let x = graph.node_x(n);
        assert!(x >= -0.01 && x <= w + 0.01);
    }
}

#[test]
fn mini_cards_when_numbers_hidden() {
    let (mut graph, _, _, ful, _, _, _) = generations();
    graph.max_descendants(1);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.anim.persons.iter().any(|p| p.base.mini));
}

#[test]
fn deep_tree_stays_finite() {
    let mut b = Builder::create();
    let mut prev: Option<PersonId> = Option::None;
    let mut ids: Vec<PersonId> = Vec::new();
    for i in 0..6 {
        let p = b.person(&format!("I{i}"), "P /D/", Some("M"), false);
        ids.push(p);
        if let Some(par) = prev {
            b.family(
                &format!("F{i}"),
                Some(par),
                Option::None,
                &[p],
                Option::None,
            );
        }
        prev = Some(p);
    }
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_ancestors(6);
    let ful = ids[5];
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn wide_sibling_row() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /E/", Some("M"), false);
    let mom = b.person("I2", "Mom /E/", Some("F"), false);
    let mut kids: Vec<PersonId> = Vec::new();
    for i in 0..6 {
        kids.push(b.person(&format!("K{i}"), "K /E/", Some("M"), false));
    }
    b.family("F1", Some(dad), Some(mom), &kids, Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run_sized(&mut graph, kids[0], 50.0, 30.0);
    assert!(all_finite(&graph));
    assert!(graph.group_count() >= 2);
}

#[test]
fn uncles_appear_with_cousins() {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /F/", Some("M"), true);
    let gm = b.person("I2", "GM /F/", Some("F"), true);
    let par = b.person("I3", "Par /F/", Some("M"), false);
    let unc = b.person("I4", "Unc /F/", Some("M"), false);
    let aunt = b.person("I5", "Aunt /F/", Some("F"), false);
    let spo = b.person("I6", "Spo /F/", Some("F"), false);
    let ful = b.person("I7", "Ful /F/", Some("M"), false);
    let cou = b.person("I8", "Cou /F/", Some("M"), false);
    b.family("F1", Some(gp), Some(gm), &[par, unc], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    b.family("F3", Some(unc), Some(aunt), &[cou], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&unc));
    assert!(ids.contains(&cou));
}

#[test]
fn half_siblings_without_spouses() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /G/", Some("M"), false);
    let m1 = b.person("I2", "M1 /G/", Some("F"), false);
    let m2 = b.person("I3", "M2 /G/", Some("F"), false);
    let ful = b.person("I4", "Ful /G/", Some("M"), false);
    let half = b.person("I5", "Half /G/", Some("F"), false);
    b.family("F1", Some(dad), Some(m1), &[ful], Option::None);
    b.family("F2", Some(dad), Some(m2), &[half], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.display_spouses(false);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn acquired_ancestry_for_partners() {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /H/", Some("M"), true);
    let gm = b.person("I2", "GM /H/", Some("F"), true);
    let inl = b.person("I3", "Inl /H/", Some("F"), false);
    let ful = b.person("I4", "Ful /H/", Some("M"), false);
    let spo = b.person("I5", "Spo /H/", Some("F"), false);
    b.family("F1", Some(gp), Some(gm), &[inl], Option::None);
    b.family("F2", Some(ful), Some(spo), &[inl], Option::None);
    b.family("F3", Some(inl), Option::None, &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
}

#[test]
fn marriage_bond_has_date_width() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /J/", Some("M"), false);
    let mom = b.person("I2", "Mom /J/", Some("F"), false);
    let k1 = b.person("I3", "K1 /J/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1], Some("1960"));
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, k1);
    let bonds = graph.anim.bond_views();
    assert!(!bonds.is_empty());
    assert!(
        bonds
            .iter()
            .any(|x| x.marriage_date.as_deref() == Some("1960"))
    );
}
