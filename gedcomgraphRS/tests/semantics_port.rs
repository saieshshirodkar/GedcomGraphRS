mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::Graph;
use gedcomgraph::model::{Fact, GPerson, GedcomData, Name};

#[test]
fn step_family_new_spouse_appears() {
    let mut b = Builder::create();
    let parent = b.person("I1", "Parent /D/", Some("M"), false);
    let spouse1 = b.person("I2", "Spouse1 /D/", Some("F"), false);
    let spouse2 = b.person("I3", "Spouse2 /D/", Some("F"), false);
    let child1 = b.person("I4", "Child1 /D/", Some("M"), false);
    b.family("F1", Some(parent), Some(spouse1), &[child1], Option::None);
    b.family("F2", Some(parent), Some(spouse2), &[], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_siblings_nephews(1);
    run(&mut graph, child1);
    assert!(all_finite(&graph));
    let ids: Vec<u32> = graph.anim.persons.iter().map(|p| p.person).collect();
    assert!(ids.contains(&spouse2));
}

#[test]
fn missing_sex_no_crash() {
    let mut b = Builder::create();
    let a = b.person("I1", "NoSex /F/", Option::None, false);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, a);
    assert!(all_finite(&graph));
    assert_eq!(
        gedcomgraph::Gender::of_person(&graph.gedcom, a),
        gedcomgraph::Gender::None
    );
}

#[test]
fn stillborn_same_date_dead() {
    let mut ged = GedcomData::empty();
    let p = ged.push_person(GPerson {
        id: "I1".to_string(),
        names: vec![Name::of("Stillborn /G/")],
        facts: vec![
            Fact::dated("BIRT", Some("1 JAN 2000")),
            Fact {
                tag: "DEAT".to_string(),
                value: Some("Y".to_string()),
                date: Some("1 JAN 2000".to_string()),
                place: Option::None,
            },
        ],
        parent_fams: Vec::new(),
        spouse_fams: Vec::new(),
    });
    ged.reindex();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, p);
    assert!(all_finite(&graph));
    let node = graph
        .anim
        .persons
        .iter()
        .find(|q| q.person == p)
        .expect("stillborn node");
    assert!(node.dead);
}

#[test]
fn family_with_only_children_no_parents() {
    let mut b = Builder::create();
    let ful = b.person("I1", "Fulcrum /H/", Some("M"), false);
    let spouse = b.person("I2", "Spouse /H/", Some("F"), false);
    let child = b.person("I3", "Child /H/", Some("M"), false);
    b.family("F1", Option::None, Option::None, &[ful], Option::None);
    b.family("F2", Some(ful), Some(spouse), &[child], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<u32> = graph.anim.persons.iter().map(|p| p.person).collect();
    assert!(ids.contains(&ful));
    assert!(ids.contains(&child));
}

#[test]
fn generation_continuity_across_multi_marriage() {
    let mut b = Builder::create();
    let mother = b.person("I1", "Mother /J/", Some("F"), false);
    let father1 = b.person("I2", "Father1 /J/", Some("M"), false);
    let father2 = b.person("I3", "Father2 /J/", Some("M"), false);
    let child1 = b.person("I4", "Child1 /J/", Some("M"), false);
    let child2 = b.person("I5", "Child2 /J/", Some("F"), false);
    b.family("F1", Some(father1), Some(mother), &[child1], Option::None);
    b.family("F2", Some(father2), Some(mother), &[child2], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, child1);
    assert!(all_finite(&graph));
    let ids: Vec<u32> = graph.anim.persons.iter().map(|p| p.person).collect();
    assert!(ids.contains(&child1));
}

#[test]
fn cousin_marriage_renders() {
    let mut b = Builder::create();
    let ggf = b.person("I1", "Grandfather /K/", Some("M"), true);
    let ggm = b.person("I2", "Grandmother /K/", Some("F"), true);
    let dad = b.person("I3", "Dad /K/", Some("M"), false);
    let aunt = b.person("I4", "Aunt /K/", Some("F"), false);
    let mom = b.person("I5", "Mom /K/", Some("F"), false);
    let uncle = b.person("I6", "Uncle /K/", Some("M"), false);
    let ful = b.person("I7", "Ful /K/", Some("M"), false);
    let spouse = b.person("I8", "Spouse /K/", Some("F"), false);
    b.family("F1", Some(ggf), Some(ggm), &[dad, aunt], Option::None);
    b.family("F2", Some(dad), Some(mom), &[ful], Option::None);
    b.family("F3", Some(uncle), Some(aunt), &[spouse], Option::None);
    b.family("F4", Some(ful), Some(spouse), &[], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<u32> = graph.anim.persons.iter().map(|p| p.person).collect();
    assert!(ids.contains(&spouse));
}

#[test]
fn solo_fulcrum_is_single_node() {
    let mut b = Builder::create();
    let a = b.person("I1", "Solo /L/", Some("M"), false);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, a);
    assert_eq!(graph.anim.person_count(), 1);
    assert!(graph.anim.persons[0].is_fulcrum());
    assert_eq!(person_generation(&graph, 0), 0);
    assert_eq!(gedcom_of(&graph, 0), a);
}

#[test]
fn five_generation_deep_positive_size() {
    let mut b = Builder::create();
    let g5 = b.person("I1", "Gen5 /A/", Some("M"), true);
    let g5sp = b.person("I2", "G5Sp /A/", Some("F"), true);
    let g4 = b.person("I3", "Gen4 /A/", Some("M"), true);
    let g4sp = b.person("I4", "G4Sp /A/", Some("F"), true);
    let g3 = b.person("I5", "Gen3 /A/", Some("M"), false);
    let g3sp = b.person("I6", "G3Sp /A/", Some("F"), false);
    let g2 = b.person("I7", "Gen2 /A/", Some("M"), false);
    let g2sp = b.person("I8", "G2Sp /A/", Some("F"), false);
    let g1 = b.person("I9", "Gen1 /A/", Some("M"), false);
    let g1sp = b.person("I10", "G1Sp /A/", Some("F"), false);
    let ful = b.person("I11", "Ful /A/", Some("M"), false);
    let spo = b.person("I12", "Spo /A/", Some("F"), false);
    let chd = b.person("I13", "Chd /A/", Some("M"), false);
    b.family("F1", Some(g5), Some(g5sp), &[g4], Option::None);
    b.family("F2", Some(g4), Some(g4sp), &[g3], Option::None);
    b.family("F3", Some(g3), Some(g3sp), &[g2], Option::None);
    b.family("F4", Some(g2), Some(g2sp), &[g1], Option::None);
    b.family("F5", Some(g1), Some(g1sp), &[ful], Option::None);
    b.family("F6", Some(ful), Some(spo), &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_ancestors(5).max_descendants(1);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.get_width() > 0.0);
    assert!(graph.get_height() > 0.0);
}

#[test]
fn cousins_marry_pedigree_collapse_duplicates() {
    let mut b = Builder::create();
    let ggp = b.person("I1", "GGP /C/", Some("M"), true);
    let ggm = b.person("I2", "GGM /C/", Some("F"), true);
    let gp1 = b.person("I3", "GP1 /C/", Some("M"), true);
    let gm1 = b.person("I4", "GM1 /C/", Some("F"), true);
    let gp2 = b.person("I5", "GP2 /C/", Some("M"), true);
    let gm2 = b.person("I6", "GM2 /C/", Some("F"), true);
    let dad = b.person("I7", "Dad /C/", Some("M"), false);
    let mom = b.person("I8", "Mom /C/", Some("F"), false);
    let spo = b.person("I9", "Spo /C/", Some("F"), false);
    let ful = b.person("I10", "Ful /C/", Some("M"), false);
    b.family("F1", Some(ggp), Some(ggm), &[gp1, gp2], Option::None);
    b.family("F2", Some(gp1), Some(gm1), &[dad], Option::None);
    b.family("F3", Some(gp2), Some(gm2), &[mom], Option::None);
    b.family("F4", Some(dad), Some(mom), &[ful], Option::None);
    b.family("F5", Some(ful), Some(spo), &[], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.display_duplicate_lines(true);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.anim.persons.iter().any(|p| p.duplicate));
}

#[test]
fn all_node_positions_unique() {
    let mut b = Builder::create();
    let par = b.person("I1", "Par /H/", Some("M"), false);
    let spo = b.person("I2", "Spo /H/", Some("F"), false);
    let ful = b.person("I3", "Ful /H/", Some("M"), false);
    let ptn = b.person("I4", "Ptn /H/", Some("F"), false);
    let chd = b.person("I5", "Chd /H/", Some("M"), false);
    b.family("F1", Some(par), Some(spo), &[ful], Option::None);
    b.family("F2", Some(ful), Some(ptn), &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run_sized(&mut graph, ful, 60.0, 40.0);
    let mut seen = std::collections::HashSet::new();
    for n in graph.anim.nodes.clone() {
        let key = (graph.node_x(n).to_bits(), graph.node_y(n).to_bits());
        assert!(seen.insert(key), "duplicate position for {n:?}");
    }
}

#[test]
fn only_one_fulcrum_node() {
    let mut b = Builder::create();
    let par = b.person("I1", "Par /I/", Some("M"), false);
    let spo = b.person("I2", "Spo /I/", Some("F"), false);
    let ful = b.person("I3", "Ful /I/", Some("M"), false);
    b.family("F1", Some(par), Some(spo), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert_eq!(
        graph.anim.persons.iter().filter(|p| p.is_fulcrum()).count(),
        1
    );
}

#[test]
fn two_marriages_two_bonds() {
    let mut b = Builder::create();
    let par = b.person("I1", "Par /G/", Some("M"), false);
    let spo = b.person("I2", "Spo /G/", Some("F"), false);
    let ful = b.person("I3", "Ful /G/", Some("M"), false);
    let ptn = b.person("I4", "Ptn /G/", Some("F"), false);
    let chd = b.person("I5", "Chd /G/", Some("M"), false);
    b.family("F1", Some(par), Some(spo), &[ful], Option::None);
    b.family("F2", Some(ful), Some(ptn), &[chd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(graph.anim.bond_views().len() >= 2);
    for x in graph.anim.bond_views() {
        assert!(x.x.is_finite() && x.y.is_finite());
    }
}

#[test]
fn mini_ancestry_cards_carry_amount() {
    let mut b = Builder::create();
    let gp = b.person("I1", "GP /D/", Some("M"), true);
    let par = b.person("I2", "Par /D/", Some("M"), false);
    let spo = b.person("I3", "Spo /D/", Some("F"), false);
    let ful = b.person("I4", "Ful /D/", Some("M"), false);
    b.family("F1", Some(gp), Option::None, &[par], Option::None);
    b.family("F2", Some(par), Some(spo), &[ful], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_ancestors(0);
    graph.display_numbers(true);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let minis: Vec<_> = graph.anim.persons.iter().filter(|p| p.base.mini).collect();
    assert!(!minis.is_empty());
    for m in minis {
        assert!(m.amount > 0);
    }
}

#[test]
fn four_gen_with_aunt_uncle() {
    let mut b = Builder::create();
    let gggp = b.person("I1", "GGGP /K/", Some("M"), true);
    let gggm = b.person("I2", "GGGM /K/", Some("F"), true);
    let ggp = b.person("I3", "GGP /K/", Some("M"), true);
    let ggm = b.person("I4", "GGM /K/", Some("F"), true);
    let gp = b.person("I5", "GP /K/", Some("M"), false);
    let gm = b.person("I6", "GM /K/", Some("F"), false);
    let par = b.person("I7", "Par /K/", Some("M"), false);
    let spo = b.person("I8", "Spo /K/", Some("F"), false);
    let aunt = b.person("I9", "Aunt /K/", Some("F"), false);
    let uncle = b.person("I10", "Unc /K/", Some("M"), false);
    let cou = b.person("I11", "Cou /K/", Some("M"), false);
    let ful = b.person("I12", "Ful /K/", Some("M"), false);
    b.family("F1", Some(gggp), Some(gggm), &[ggp], Option::None);
    b.family("F2", Some(ggp), Some(ggm), &[gp], Option::None);
    b.family("F3", Some(gp), Some(gm), &[par, aunt], Option::None);
    b.family("F4", Some(par), Some(spo), &[ful], Option::None);
    b.family("F5", Some(uncle), Some(aunt), &[cou], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_ancestors(4).max_uncles_cousins(2);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<u32> = graph.anim.persons.iter().map(|p| p.person).collect();
    assert!(ids.contains(&aunt));
    assert!(ids.contains(&cou));
}

#[test]
fn progeny_mini_sits_below_parent() {
    let mut b = Builder::create();
    let ful = b.person("I1", "Ful /A/", Some("M"), false);
    let spo = b.person("I2", "Spo /A/", Some("F"), false);
    let chd = b.person("I3", "Chd /A/", Some("M"), false);
    let ptn = b.person("I4", "Ptn /A/", Some("F"), false);
    let grd = b.person("I5", "Grd /A/", Some("M"), false);
    b.family("F1", Some(ful), Some(spo), &[chd], Option::None);
    b.family("F2", Some(chd), Some(ptn), &[grd], Option::None);
    let ged = b.finish();
    let mut graph = Graph::with_gedcom(ged);
    graph.max_descendants(1);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let mut found = false;
    for (i, p) in graph.anim.persons.iter().enumerate() {
        if !p.base.mini {
            continue;
        }
        let Some(o) = p.origin else { continue };
        let (origin_mini, oy) = match o {
            gedcomgraph::core::NodeId::Person(q) => {
                let q = &graph.anim.persons[q as usize];
                (q.base.mini, q.base.y)
            }
            gedcomgraph::core::NodeId::Family(q) => {
                let q = &graph.anim.families[q as usize];
                (q.base.mini, q.base.y)
            }
        };
        if origin_mini {
            continue;
        }
        found = true;
        assert!(
            p.base.y > oy,
            "progeny mini {i} y {} below origin y {oy}",
            p.base.y
        );
    }
    assert!(found, "expected a progeny mini with regular origin");
}
