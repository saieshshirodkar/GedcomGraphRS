mod common;

use common::{all_finite, run};
use gedcomgraph::GedcomData;
use gedcomgraph::model::helpers::{family_restricted, person_restricted};
use gedcomgraph::parse_gedcom;

fn restricted_fixture() -> GedcomData {
    parse_gedcom(
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n\
         0 @I1@ INDI\n1 NAME Dad /R/\n1 SEX M\n1 RESN CONFIDENTIAL\n1 FAMC @F1@\n\
         0 @I2@ INDI\n1 NAME Kid /R/\n1 SEX M\n1 FAMC @F1@\n\
         0 @F1@ FAM\n1 HUSB @I1@\n1 CHIL @I2@\n1 MARR\n2 DATE 1900\n0 TRLR\n",
    )
}

#[test]
fn restricted_flags_parsed() {
    let ged = restricted_fixture();
    let dad = ged.find_person("I1").expect("dad");
    let kid = ged.find_person("I2").expect("kid");
    assert!(person_restricted(&ged, dad.0));
    assert!(!person_restricted(&ged, kid.0));
}

#[test]
fn restricted_person_renders_placeholder() {
    let ged = restricted_fixture();
    let dad = ged.find_person("I1").expect("dad");
    let lines = gedcomgraph::render::texts::name_lines(&ged, dad.0, false);
    assert_eq!(lines, vec!["[Restricted]".to_string()]);
    assert!(gedcomgraph::render::texts::date_lines(&ged, dad.0).is_empty());
    assert_eq!(gedcomgraph::render::texts::titles_of(&ged, dad.0), "");
}

#[test]
fn restricted_family_marriage_date_hidden() {
    let ged = parse_gedcom(
        "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n\
         0 @I1@ INDI\n1 NAME Dad /R/\n1 SEX M\n1 FAMS @F1@\n\
         0 @I2@ INDI\n1 NAME Mum /R/\n1 SEX F\n1 FAMS @F1@\n\
         0 @I3@ INDI\n1 NAME Kid /R/\n1 SEX M\n1 FAMC @F1@\n\
         0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 CHIL @I3@\n1 MARR\n2 DATE 1900\n\
         1 RESN PRIVACY\n0 TRLR\n",
    );
    let fam = ged.find_family("F1").expect("fam");
    assert!(family_restricted(&ged, fam.0));
    assert_eq!(
        gedcomgraph::model::kin::family_marriage_date(&ged, fam.0),
        Option::None
    );
}

#[test]
fn restricted_run_stays_finite() {
    let ged = restricted_fixture();
    let kid = ged.find_person("I2").expect("kid");
    let mut graph = gedcomgraph::Graph::with_gedcom(ged);
    graph.set_max_bitmap_size(2000.0);
    run(&mut graph, kid);
    assert!(all_finite(&graph));
}

#[test]
fn unrestricted_person_keeps_details() {
    let ged = restricted_fixture();
    let kid = ged.find_person("I2").expect("kid");
    let lines = gedcomgraph::render::texts::name_lines(&ged, kid.0, false);
    assert_eq!(lines, vec!["Kid".to_string()]);
}
