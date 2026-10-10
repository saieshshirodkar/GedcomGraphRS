mod common;

use common::Builder;
use gedcomgraph::model::{FamilyId, GFamily, GPerson, Name, PersonId};
use gedcomgraph::{GedcomData, Graph};

fn person(id: &str) -> GPerson {
    GPerson {
        id: id.to_string(),
        names: vec![Name::of(id)],
        facts: Vec::new(),
        parent_fams: Vec::new(),
        spouse_fams: Vec::new(),
        ..GPerson::default()
    }
}

#[test]
fn push_invalidates_lookup_order() {
    let mut g = GedcomData::empty();
    g.push_person(person("I1"));
    g.reindex();
    assert_eq!(g.find_person("I1"), Some(PersonId(0)));

    g.push_person(person("I0"));
    assert_eq!(g.find_person("I0"), None, "stale order must not answer");

    g.reindex();
    assert_eq!(g.find_person("I0"), Some(PersonId(1)));
    assert_eq!(g.find_person("I1"), Some(PersonId(0)));
}

#[test]
fn push_family_invalidates_family_order() {
    let mut g = GedcomData::empty();
    g.push_family(GFamily {
        id: "F1".to_string(),
        husbands: Vec::new(),
        wives: Vec::new(),
        children: Vec::new(),
        facts: Vec::new(),
        ..GFamily::default()
    });
    g.reindex();
    assert_eq!(g.find_family("F1"), Some(FamilyId(0)));
    g.push_family(GFamily {
        id: "F0".to_string(),
        husbands: Vec::new(),
        wives: Vec::new(),
        children: Vec::new(),
        facts: Vec::new(),
        ..GFamily::default()
    });
    assert_eq!(g.find_family("F0"), None);
    g.reindex();
    assert_eq!(g.find_family("F0"), Some(FamilyId(1)));
}

#[test]
fn lookup_survives_many_unsorted_pushes() {
    let mut g = GedcomData::empty();
    for id in ["I5", "I1", "I9", "I3", "I7", "I0", "I2"] {
        g.push_person(person(id));
    }
    g.reindex();
    assert_eq!(g.find_person("I0"), Some(PersonId(5)));
    assert_eq!(g.find_person("I9"), Some(PersonId(2)));
    assert_eq!(g.find_person("I4"), None);
    assert_eq!(g.find_person(""), None);
}

#[test]
fn missing_lookup_is_none_not_panic() {
    let g = GedcomData::empty();
    assert_eq!(g.find_person("I1"), None);
    assert_eq!(g.find_family("F1"), None);
    let mut b = Builder::create();
    b.person("I1", "A /B/", Some("M"), false);
    let ged = b.finish();
    assert_eq!(ged.find_person("ZZ"), None);
}

#[test]
fn normalized_width_is_zero_for_empty_run() {
    let mut g = Graph::with_gedcom(GedcomData::empty());
    g.start_from(0);
    g.init_nodes();
    g.place_nodes();
    assert_eq!((g.width(), g.height()), (0.0, 0.0));
    assert!(g.width().is_finite() && g.height().is_finite());
}

#[test]
fn width_stays_finite_for_real_tree() {
    let path = format!("{}/example.ged", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).expect("example.ged");
    let ged = gedcomgraph::parse_gedcom(&text);
    let ful = ged.find_person("I3").expect("fulcrum").0;
    let mut g = Graph::with_gedcom(ged);
    g.start_from(ful);
    g.init_nodes();
    g.place_nodes();
    assert!(g.width() > 0.0);
    assert!(g.height() > 0.0);
    assert!(g.width().is_finite() && g.height().is_finite());
}
