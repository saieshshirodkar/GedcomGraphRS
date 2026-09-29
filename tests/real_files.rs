mod common;

use common::{Builder, all_finite, gedcom_of, person_generation, run, run_sized};
use gedcomgraph::model::{Fact, GPerson, GedcomData, Name, PersonId};
use gedcomgraph::{Graph, parse_gedcom};

fn load(path: &str) -> Graph {
    let text = std::fs::read_to_string(path).expect("read gedcom");
    Graph::with_gedcom(parse_gedcom(&text))
}

#[test]
fn example_ged_smoke() {
    let mut graph = load("example.ged");
    let ful = graph.gedcom.find_person("I3").expect("fulcrum");
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, ful);
    assert!(graph.anim.node_count() >= 5);
    assert!(graph.anim.person_count() >= 7);
    assert!(all_finite(&graph));
    assert!(graph.width() > 0.0);
    assert!(graph.height() > 0.0);
    assert!(!graph.anim.bond_views().is_empty());
    assert!(!graph.anim.line_groups.is_empty());
}

#[test]
fn example_all_persons_reachable() {
    let mut graph = load("example.ged");
    for target in ["I1", "I3", "I6"] {
        let ful = graph.gedcom.find_person(target).expect("person");
        run(&mut graph, ful);
        assert!(all_finite(&graph), "finite from {target}");
    }
    for f in graph.fulcrum_persons() {
        let _ = person_generation(&graph, f);
        let _ = gedcom_of(&graph, f);
    }
}

#[test]
fn builder_mirrors_example_structure() {
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
    let mut graph = Graph::with_gedcom(b.finish());
    run_sized(&mut graph, cain, 60.0, 40.0);
    assert!(all_finite(&graph));
    assert!(graph.anim.person_count() >= 7);
}

#[test]
fn demo_ged_smoke() {
    let mut graph = load("demo.ged");
    assert!(graph.gedcom.person_count() >= 10);
    let ful = graph.gedcom.find_person("I14").unwrap_or(PersonId(0));
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, ful);
    assert!(graph.anim.node_count() >= 5);
    assert!(all_finite(&graph));
    assert!(graph.width().is_finite());
    assert!(graph.height().is_finite());
}

#[test]
fn demo_every_person_as_fulcrum() {
    let base = load("demo.ged");
    let ids: Vec<PersonId> = (0..u32::try_from(base.gedcom.person_count()).unwrap_or(u32::MAX))
        .map(PersonId)
        .collect();
    for ful in ids {
        let mut graph = load("demo.ged");
        run(&mut graph, ful);
        assert!(all_finite(&graph));
    }
}

#[test]
fn example_every_person_as_fulcrum() {
    let base = load("example.ged");
    let ids: Vec<PersonId> = (0..u32::try_from(base.gedcom.person_count()).unwrap_or(u32::MAX))
        .map(PersonId)
        .collect();
    assert!(ids.len() >= 50);
    for ful in ids {
        let mut graph = load("example.ged");
        run(&mut graph, ful);
        assert!(all_finite(&graph), "finite from fulcrum {ful}");
        assert!(graph.width() >= 0.0);
        assert!(graph.height() >= 0.0);
    }
}

#[test]
fn edge_solo_person_no_relatives() {
    let ged = parse_gedcom("0 HEAD\n0 @I1@ INDI\n1 NAME Solo /Person/\n0 TRLR\n");
    let ful = ged.find_person("I1").expect("fulcrum");
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    assert!(graph.anim.node_count() >= 1);
}

#[test]
fn edge_two_person_parent_child() {
    let mut raw = "0 HEAD\n0 @I1@ INDI\n1 NAME Parent /\n1 FAMS @F1@\n".to_string();
    raw.push_str("0 @I2@ INDI\n1 NAME Child /\n1 FAMC @F1@\n");
    raw.push_str("0 @F1@ FAM\n1 HUSB @I1@\n1 CHIL @I2@\n0 TRLR\n");
    let ged = parse_gedcom(&raw);
    let ful = ged.find_person("I2").expect("child");
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&PersonId(0)));
}

#[test]
fn edge_multi_marriage_without_children() {
    let mut raw = "0 HEAD\n0 @I1@ INDI\n1 NAME Person /\n1 SEX M\n".to_string();
    raw.push_str("1 FAMS @F1@\n1 FAMS @F2@\n");
    raw.push_str("0 @I2@ INDI\n1 NAME Spouse1 /\n1 SEX F\n1 FAMS @F1@\n");
    raw.push_str("0 @I3@ INDI\n1 NAME Spouse2 /\n1 SEX F\n1 FAMS @F2@\n");
    raw.push_str("0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n");
    raw.push_str("0 @F2@ FAM\n1 HUSB @I1@\n1 WIFE @I3@\n0 TRLR\n");
    let ged = parse_gedcom(&raw);
    let ful = ged.find_person("I1").expect("person");
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let ids: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    assert!(ids.contains(&ful));
}

#[test]
fn edge_same_birth_and_death_dates() {
    let mut ged = GedcomData::empty();
    let ful = ged.push_person(GPerson {
        id: "I1".to_string(),
        names: vec![Name::of("Still /Born/")],
        facts: vec![
            Fact::dated("BIRT", Some("1 JAN 2000")),
            Fact::dated("DEAT", Some("1 JAN 2000")),
        ],
        parent_fams: Vec::new(),
        spouse_fams: Vec::new(),
    });
    ged.reindex();
    let mut graph = Graph::with_gedcom(ged);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    let node = graph
        .anim
        .persons
        .iter()
        .find(|p| PersonId(p.person) == ful)
        .expect("node");
    assert!(node.dead);
}

#[test]
fn example_full_depth_shows_all_descendants() {
    let text = std::fs::read_to_string("example.ged").expect("read gedcom");
    let ged = parse_gedcom(&text);
    let ful = ged.find_person("I3").expect("fulcrum");
    let mut graph = Graph::with_gedcom(ged);
    graph
        .max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    for id in [
        "I32", "I33", "I34", "I36", "I37", "I39", "I41", "I43", "I47", "I48", "I55", "I57",
    ] {
        let p = graph.gedcom.find_person(id).expect("person");
        let node = graph
            .anim
            .persons
            .iter()
            .find(|q| PersonId(q.person) == p)
            .expect("node");
        assert!(!node.base.mini, "{id} renders as full card");
    }
}

#[test]
fn example_inlaw_parents_hidden() {
    let text = std::fs::read_to_string("example.ged").expect("read gedcom");
    let ged = parse_gedcom(&text);
    let ful = ged.find_person("I3").expect("fulcrum");
    let mut graph = Graph::with_gedcom(ged);
    graph
        .max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10);
    run(&mut graph, ful);
    assert!(all_finite(&graph));
    for id in ["I18", "I19", "I20", "I21"] {
        let p = graph.gedcom.find_person(id).expect("person");
        assert!(
            graph.anim.persons.iter().all(|q| PersonId(q.person) != p),
            "{id} has no card"
        );
    }
}
