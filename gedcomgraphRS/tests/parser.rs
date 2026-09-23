use gedcomgraph::parse_gedcom;

const SIMPLE: &str = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 @I1@ INDI\n1 NAME John /Smith/\n1 SEX M\n1 BIRT\n2 DATE 1870\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Mary /Doe/\n1 SEX F\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME Kid /Smith/\n1 SEX M\n1 FAMC @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 CHIL @I3@\n1 MARR\n2 DATE 1895\n0 TRLR\n";

#[test]
fn parses_people_and_family() {
    let g = parse_gedcom(SIMPLE);
    assert_eq!(g.person_count(), 3);
    assert_eq!(g.family_count(), 1);
}

#[test]
fn strips_at_signs() {
    let g = parse_gedcom(SIMPLE);
    assert_eq!(g.find_person("I1"), Some(0));
    assert_eq!(g.find_family("F1"), Some(0));
    assert_eq!(g.find_person("ZZZ"), Option::None);
}

#[test]
fn links_relations() {
    let g = parse_gedcom(SIMPLE);
    let f = g.family(0).expect("family");
    assert_eq!(f.husbands, vec![0]);
    assert_eq!(f.wives, vec![1]);
    assert_eq!(f.children, vec![2]);
    assert_eq!(g.person(2).expect("kid").parent_fams, vec![0]);
}

#[test]
fn keeps_facts() {
    let g = parse_gedcom(SIMPLE);
    let p = g.person(0).expect("person");
    assert!(p.facts.iter().any(|f| f.tag == "SEX"));
    assert!(p.facts.iter().any(|f| f.tag == "BIRT"));
    let f = g.family(0).expect("family");
    let m = f.facts.iter().find(|e| e.tag == "MARR").expect("marr");
    assert_eq!(m.date.as_deref(), Some("1895"));
}

#[test]
fn empty_input() {
    let g = parse_gedcom("");
    assert_eq!(g.person_count(), 0);
    assert_eq!(g.family_count(), 0);
}

#[test]
fn dangling_refs_ignored() {
    let txt =
        "0 @I1@ INDI\n1 NAME Solo\n1 FAMS @F9@\n0 @F9@ FAM\n1 HUSB @I1@\n1 WIFE @I7@\n0 TRLR\n";
    let g = parse_gedcom(txt);
    assert_eq!(g.person_count(), 1);
    assert_eq!(g.family_count(), 1);
    assert!(g.family(0).expect("fam").wives.is_empty());
}

#[test]
fn birth_place_attached() {
    let txt = "0 @I1@ INDI\n1 NAME A /B/\n1 BIRT\n2 DATE 1870\n2 PLAC Oslo\n0 TRLR\n";
    let g = parse_gedcom(txt);
    let p = g.person(0).expect("person");
    let b = p.facts.iter().find(|f| f.tag == "BIRT").expect("birt");
    assert_eq!(b.date.as_deref(), Some("1870"));
    assert_eq!(b.place.as_deref(), Some("Oslo"));
}

#[test]
fn unknown_tag_preserved_as_fact() {
    let txt = "0 @I1@ INDI\n1 NAME A /B/\n1 OCCU Blacksmith\n2 DATE 1900\n0 TRLR\n";
    let g = parse_gedcom(txt);
    let p = g.person(0).expect("person");
    let o = p.facts.iter().find(|f| f.tag == "OCCU").expect("occu");
    assert_eq!(o.value.as_deref(), Some("Blacksmith"));
    assert_eq!(o.date.as_deref(), Some("1900"));
}

#[test]
fn marriage_place_kept() {
    let txt = "0 @F1@ FAM\n1 MARR\n2 DATE 1895\n2 PLAC Bergen\n0 TRLR\n";
    let g = parse_gedcom(txt);
    let f = g.family(0).expect("family");
    let m = f.facts.iter().find(|e| e.tag == "MARR").expect("marr");
    assert_eq!(m.date.as_deref(), Some("1895"));
    assert_eq!(m.place.as_deref(), Some("Bergen"));
}

#[test]
fn garbage_lines_ignored() {
    let txt = "HELLO\n\n   \n0 HEAD\nNOT A LEVEL\n1\n0 @I1@ INDI\n1 NAME A\n0 TRLR\nbogus\n";
    let g = parse_gedcom(txt);
    assert_eq!(g.person_count(), 1);
    assert_eq!(g.family_count(), 0);
}

#[test]
fn multiple_names_kept() {
    let txt = "0 @I1@ INDI\n1 NAME First /A/\n1 NAME Second /B/\n0 TRLR\n";
    let g = parse_gedcom(txt);
    assert_eq!(g.person(0).expect("person").names.len(), 2);
}

#[test]
fn repeated_links_deduplicated() {
    let txt = "0 @I1@ INDI\n1 NAME A\n1 FAMS @F1@\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME B\n1 FAMS @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 HUSB @I1@\n1 WIFE @I2@\n0 TRLR\n";
    let g = parse_gedcom(txt);
    assert_eq!(g.person(0).expect("a").spouse_fams.len(), 1);
    assert_eq!(g.family(0).expect("fam").husbands.len(), 1);
    assert_eq!(g.find_family("F1"), Some(0));
    assert_eq!(g.find_family("ZZZ"), Option::None);
}
