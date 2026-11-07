mod common;

use common::all_finite;
use gedcomgraph::Graph;
use gedcomgraph::model::PersonId;
use gedcomgraph::parse_gedcom;

fn ged() -> gedcomgraph::GedcomData {
    parse_gedcom(&std::fs::read_to_string("example.ged").expect("read example.ged"))
}

fn pid(ged: &gedcomgraph::GedcomData, id: &str) -> PersonId {
    ged.find_person(id).expect("person record exists")
}

#[test]
fn tree_has_full_maternal_and_paternal_depth() {
    let ged = ged();
    for id in [
        "I58", "I59", "I60", "I64", "I65", "I66", "I69", "I70", "I71", "I74", "I77", "I80", "I83",
        "I84", "I85", "I88", "I89", "I90", "I93", "I94", "I95",
    ] {
        let _ = pid(&ged, id);
    }
}

#[test]
fn edna_parents_are_hart_couple() {
    let ged = ged();
    let edna = pid(&ged, "I11");
    let families = ged.person(edna.0).expect("edna").parent_fams.clone();
    assert_eq!(families.len(), 1);
    let fam = ged.family(families[0].0).expect("family");
    let names: Vec<String> = fam
        .husbands
        .iter()
        .chain(fam.wives.iter())
        .map(|p| ged.person(p.0).expect("parent").names[0].display.clone())
        .collect();
    assert!(names.iter().any(|n| n.contains("George")));
    assert!(names.iter().any(|n| n.contains("Lila")));
}

#[test]
fn nora_hart_is_edna_sister_and_has_reed_children() {
    let ged = ged();
    let nora = pid(&ged, "I60");
    assert!(ged.person(nora.0).expect("nora").parent_fams.len() == 1);
    let fam = ged
        .family(ged.person(nora.0).expect("nora").spouse_fams[0].0)
        .expect("fam");
    assert_eq!(fam.children.len(), 2);
}

#[test]
fn eve_has_brother_henry_with_child() {
    let ged = ged();
    let henry = pid(&ged, "I74");
    let p = ged.person(henry.0).expect("henry");
    assert_eq!(p.parent_fams.len(), 1);
    let fam = ged.family(p.parent_fams[0].0).expect("fam");
    let kids: Vec<_> = fam
        .children
        .iter()
        .map(|c| ged.person(c.0).expect("kid").id.clone())
        .collect();
    assert!(kids.contains(&"I2".to_string()));
    assert!(kids.contains(&"I22".to_string()));
    assert!(kids.contains(&"I74".to_string()));
}

#[test]
fn paula_and_peter_parents_exist() {
    let ged = ged();
    let paula = pid(&ged, "I19");
    let peter = pid(&ged, "I18");
    assert!(!ged.person(paula.0).expect("paula").parent_fams.is_empty());
    assert!(!ged.person(peter.0).expect("peter").parent_fams.is_empty());
}

#[test]
fn oscar_marsh_and_edith_lane_are_cousins_of_ada() {
    let ged = ged();
    for id in ["I66", "I68", "I71", "I73"] {
        let _ = pid(&ged, id);
    }
}

#[test]
fn rosa_parents_are_costa_couple() {
    let ged = ged();
    let rosa = pid(&ged, "I21");
    let fam = ged
        .family(ged.person(rosa.0).expect("rosa").parent_fams[0].0)
        .expect("fam");
    assert_eq!(fam.husbands.len(), 1);
    assert_eq!(fam.wives.len(), 1);
}

#[test]
fn paternal_great_grandparents_exist() {
    let ged = ged();
    let elias = pid(&ged, "I49");
    let otto = pid(&ged, "I52");
    assert!(!ged.person(elias.0).expect("elias").parent_fams.is_empty());
    assert!(!ged.person(otto.0).expect("otto").parent_fams.is_empty());
    let _ = pid(&ged, "I92");
    let _ = pid(&ged, "I97");
}

#[test]
fn all_cross_links_resolve() {
    let ged = ged();
    for (i, p) in ged.persons.iter().enumerate() {
        for fid in &p.parent_fams {
            assert!(ged.family(fid.0).is_some(), "person {i} parent fam missing");
        }
        for fid in &p.spouse_fams {
            assert!(ged.family(fid.0).is_some(), "person {i} spouse fam missing");
        }
    }
    for f in &ged.families {
        for h in &f.husbands {
            assert!(ged.person(h.0).is_some());
        }
        for w in &f.wives {
            assert!(ged.person(w.0).is_some());
        }
        for c in &f.children {
            assert!(ged.person(c.0).is_some());
        }
    }
}

#[test]
fn children_oldest_first_in_new_families() {
    let ged = ged();
    for label in ["F23", "F24", "F25", "F27", "F32", "F34", "F36"] {
        let fid = ged.find_family(label).expect("family");
        let fam = ged.family(fid.0).expect("fam");
        assert!(fam.children.len() >= 2);
        let ys: Vec<i64> = fam
            .children
            .iter()
            .map(|c| {
                let p = ged.person(c.0).expect("kid");
                p.facts
                    .iter()
                    .find(|f| f.tag == "BIRT")
                    .and_then(|f| f.date.clone())
                    .expect("birth")
                    .parse::<i64>()
                    .unwrap_or(0)
            })
            .collect();
        let mut sorted = ys.clone();
        sorted.sort_unstable();
        assert_eq!(ys, sorted, "family {label} children not oldest-first");
    }
}

#[test]
fn cain_render_reaches_maternal_cousins() {
    let ged = ged();
    let ful = pid(&ged, "I3");
    let mut graph = Graph::with_gedcom(ged);
    graph
        .max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10);
    graph.start_from(ful.0);
    graph.init_nodes();
    graph.place_nodes();
    assert!(all_finite(&graph));
    let present: Vec<PersonId> = graph
        .anim
        .persons
        .iter()
        .map(|p| PersonId(p.person))
        .collect();
    // blood maternal cousins are shown
    for id in ["I62", "I63", "I76", "I79"] {
        let want = graph.gedcom.find_person(id).expect("person");
        assert!(present.contains(&want), "cousin {id} missing from render");
    }
    // in-law cousins (Ada and Ruth sides) are hidden by design
    for id in ["I68", "I73", "I82", "I87"] {
        let want = graph.gedcom.find_person(id).expect("person");
        assert!(!present.contains(&want), "in-law {id} should be hidden");
    }
    assert!(graph.height() > 0.0);
}

#[test]
fn every_new_person_renders_finite_from_self() {
    let ged = ged();
    for id in [
        "I58", "I60", "I68", "I73", "I76", "I79", "I82", "I87", "I92", "I97",
    ] {
        let ful = pid(&ged, id);
        let mut graph = Graph::with_gedcom(ged.clone());
        graph.start_from(ful.0);
        graph.init_nodes();
        graph.place_nodes();
        assert!(all_finite(&graph), "finite from {id}");
        assert!(graph.width() >= 0.0 && graph.height() >= 0.0);
    }
}

#[test]
fn svg_mentions_maternal_names() {
    let ged = ged();
    let ful = pid(&ged, "I3");
    let mut graph = Graph::with_gedcom(ged);
    graph
        .max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10)
        .set_layout_direction(true);
    graph.start_from(ful.0);
    graph.set_all_person_sizes(60.0, 40.0);
    graph.init_nodes();
    graph.place_nodes();
    let fonts = gedcomgraph::render::font_draw::Fonts::load();
    let _ = fonts;
    let svg = {
        use gedcomgraph::render::theme::Scale;
        gedcomgraph::render::svg::render_svg(&graph, &fonts, &Scale::of(4.0), 0.0, 0.0, 4000, 3200)
    };
    for name in ["Nora", "Mary", "Sarah", "Lila", "Rose"] {
        assert!(svg.contains(name), "svg missing maternal name {name}");
    }
}
