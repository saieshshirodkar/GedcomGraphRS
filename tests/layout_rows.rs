mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Branch;
use gedcomgraph::core::NodeId;
use gedcomgraph::model::PersonId;

fn three_gen() -> (
    gedcomgraph::Graph,
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
    b.family("F3", Some(ful), Some(spo), &[chd], Option::None);
    let ged = b.finish();
    (
        gedcomgraph::Graph::with_gedcom(ged),
        gp,
        gm,
        par,
        spo,
        ful,
        chd,
    )
}

fn is_mini(graph: &gedcomgraph::Graph, nid: NodeId) -> bool {
    match nid {
        NodeId::Person(i) => graph.anim.persons[i as usize].base.mini,
        NodeId::Family(i) => graph.anim.families[i as usize].base.mini,
    }
}

fn person_count(graph: &gedcomgraph::Graph, nid: NodeId) -> usize {
    match nid {
        NodeId::Person(i) => {
            if (i as usize) < graph.anim.persons.len() {
                1
            } else {
                0
            }
        }
        NodeId::Family(i) => graph.anim.families[i as usize].partners.len(),
    }
}

#[test]
fn union_rows_cover_nonmini_nodes() {
    let (mut graph, _, _, _, _, _, ful) = three_gen();
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(!graph.anim.union_rows.is_empty());
    for nid in graph.anim.nodes.clone() {
        if is_mini(&graph, nid) || person_count(&graph, nid) == 0 {
            continue;
        }
        let covered = graph.anim.unions.iter().any(|u| u.list.contains(&nid));
        assert!(covered, "{nid:?} has a union");
    }
}

#[test]
fn grouprows_sorted_by_generation() {
    let (mut graph, _, _, _, _, _, ful) = three_gen();
    run(&mut graph, ful);
    let rows = &graph.anim.group_rows;
    assert!(!rows.is_empty());
    for w in rows.windows(2) {
        assert!(w[0].generation < w[1].generation);
    }
    let urows = &graph.anim.union_rows;
    for w in urows.windows(2) {
        assert!(w[0].generation < w[1].generation);
    }
}

#[test]
fn union_prev_next_chains() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /C/", Some("M"), false);
    let mom = b.person("I2", "Mom /C/", Some("F"), false);
    let ful = b.person("I3", "Ful /C/", Some("M"), false);
    let sib = b.person("I4", "Sib /C/", Some("F"), false);
    let unc = b.person("I5", "Unc /C/", Some("M"), false);
    let aunt = b.person("I6", "Aunt /C/", Some("F"), false);
    b.family("F1", Some(dad), Some(mom), &[ful, sib], Option::None);
    b.family("F2", Some(unc), Some(aunt), &[], Option::None);
    let mut graph = gedcomgraph::Graph::with_gedcom(b.finish());
    graph.max_siblings_nephews(1);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    for row in &graph.anim.union_rows {
        for w in row.unions.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert_eq!(graph.anim.unions[a as usize].next, Some(b));
            assert_eq!(graph.anim.unions[b as usize].prev, Some(a));
        }
    }
}

#[test]
fn two_parent_group_has_pater_branch() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /E/", Some("M"), false);
    let mom = b.person("I2", "Mom /E/", Some("F"), false);
    let ful = b.person("I3", "Ful /E/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[ful], Option::None);
    let mut graph = gedcomgraph::Graph::with_gedcom(b.finish());
    run(&mut graph, ful);
    let mut found = false;
    for row in &graph.anim.group_rows {
        for gid in &row.groups {
            let g = &graph.anim.groups[*gid as usize];
            if g.generation < 0 && !g.list.is_empty() {
                assert!(
                    g.branch == Branch::Pater
                        || g.branch == Branch::Mater
                        || g.branch == Branch::None
                );
                if g.branch == Branch::Pater {
                    found = true;
                }
            }
        }
    }
    assert!(found, "a pater ancestor group exists");
}

#[test]
fn youth_groups_nonempty_finite() {
    let (mut graph, _, _, _, _, _, ful) = three_gen();
    run_sized(&mut graph, ful, 60.0, 40.0);
    assert!(all_finite(&graph));
    for g in &graph.anim.groups {
        assert!(!g.list.is_empty());
        for nid in &g.list {
            assert!(graph.node_x(*nid).is_finite());
            assert!(graph.node_y(*nid).is_finite());
        }
    }
}

#[test]
fn origins_finite() {
    let (mut graph, _, _, _, _, _, ful) = three_gen();
    run(&mut graph, ful);
    for p in &graph.anim.persons {
        if let Some(o) = p.origin {
            assert!(graph.node_x(o).is_finite());
            assert!(graph.node_y(o).is_finite());
        }
    }
}

#[test]
fn sibling_row_has_no_overlap() {
    let mut b = Builder::create();
    let dad = b.person("I1", "Dad /B/", Some("M"), false);
    let mom = b.person("I2", "Mom /B/", Some("F"), false);
    let k1 = b.person("I3", "K1 /B/", Some("M"), false);
    let k2 = b.person("I4", "K2 /B/", Some("F"), false);
    let k3 = b.person("I5", "K3 /B/", Some("M"), false);
    b.family("F1", Some(dad), Some(mom), &[k1, k2, k3], Option::None);
    let mut graph = gedcomgraph::Graph::with_gedcom(b.finish());
    run_sized(&mut graph, k1, 60.0, 40.0);
    let mut kids: Vec<(f32, f32)> = graph
        .anim
        .persons
        .iter()
        .filter(|p| p.base.generation == 0)
        .map(|p| (p.base.x, p.base.w))
        .collect();
    kids.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for w in kids.windows(2) {
        assert!(w[0].0 + w[0].1 <= w[1].0 + 0.01, "overlap {w:?}");
    }
}

#[test]
fn fulcrum_lives_in_ancestor_union() {
    let (mut graph, _, _, _, _, _, ful) = three_gen();
    run(&mut graph, ful);
    let fi = graph
        .anim
        .persons
        .iter()
        .position(|p| p.is_fulcrum())
        .expect("fulcrum") as u32;
    let nid = match graph.anim.persons[fi as usize].family {
        Some(f) => NodeId::Family(f),
        Option::None => NodeId::Person(fi),
    };
    assert_eq!(person_generation(&graph, fi), 0);
    assert_eq!(
        graph
            .gedcom
            .find_person(&graph.person_gedcom_id(fi))
            .unwrap_or(PersonId(u32::MAX)),
        gedcom_of(&graph, fi)
    );
    assert!(
        graph
            .anim
            .unions
            .iter()
            .any(|u| u.ancestor == Some(nid) || u.list.contains(&nid)),
        "fulcrum placed in a union"
    );
}
