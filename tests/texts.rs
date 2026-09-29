#[cfg(test)]
mod tests {
    use gedcomgraph::model::PersonId;
    use gedcomgraph::model::gedcom::{Fact, GPerson, GedcomData, Name};
    use gedcomgraph::render::texts::*;

    fn ged_one(display: &str, facts: Vec<Fact>) -> (GedcomData, PersonId) {
        let mut g = GedcomData::empty();
        let p = g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of(display)],
            facts,
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.reindex();
        (g, p)
    }

    #[test]
    fn given_name_rules() {
        assert_eq!(given_name("John /Smith/"), "John");
        assert_eq!(given_name("/Smith/ Jr"), "Smith");
        assert_eq!(given_name("/Smith/"), "Smith");
        assert_eq!(given_name("Plato"), "Plato");
        assert_eq!(given_name(""), "[No name]");
        assert_eq!(given_name("  Ann  /X/ "), "Ann");
    }

    #[test]
    fn name_lines_and_duplicate() {
        let (g, p) = ged_one("John /Smith/", Vec::new());
        assert_eq!(name_lines(&g, p.0, false), vec!["John".to_string()]);
        assert_eq!(name_lines(&g, p.0, true), vec!["John (2)".to_string()]);
        assert_eq!(name_lines(&g, 9, false), vec!["[No name]".to_string()]);
    }

    #[test]
    fn titles_join() {
        let (g, p) = ged_one(
            "A /B/",
            vec![
                Fact::of("TITL", Some("King"), Option::None),
                Fact::of("TITL", Some("Wise"), Option::None),
                Fact::of("SEX", Some("M"), Option::None),
            ],
        );
        assert_eq!(titles_of(&g, p.0), "King\nWise");
        assert_eq!(titles_of(&g, 9), "");
    }

    #[test]
    fn dead_and_dates() {
        let (g, p) = ged_one(
            "A /B/",
            vec![
                Fact::dated("BIRT", Some("1870")),
                Fact::dated("DEAT", Some("1940")),
            ],
        );
        assert!(is_dead(&g, p.0));
        assert!(!is_dead(&g, 9));
        assert_eq!(
            date_lines(&g, p.0),
            vec!["\u{2605} 1870  \u{271B} 1940".to_string()]
        );
    }

    #[test]
    fn short_dates_share_line() {
        let (g, p) = ged_one(
            "A /B/",
            vec![
                Fact::dated("BIRT", Some("70")),
                Fact::dated("DEAT", Some("40")),
            ],
        );
        assert_eq!(
            date_lines(&g, p.0),
            vec!["\u{2605} 70  \u{271B} 40".to_string()]
        );
    }

    #[test]
    fn christening_fallback() {
        let (g, p) = ged_one("A /B/", vec![Fact::dated("CHR", Some("1871"))]);
        assert_eq!(date_lines(&g, p.0), vec!["\u{2248} 1871".to_string()]);
        let (h, q) = ged_one("A /B/", vec![Fact::dated("OCCU", Some("Smith"))]);
        assert_eq!(date_lines(&h, q.0), vec!["Smith".to_string()]);
        let (j, r) = ged_one("A /B/", vec![Fact::of("OCCU", Some("Smith"), Option::None)]);
        assert!(date_lines(&j, r.0).is_empty());
    }

    #[test]
    fn bapm_fallback() {
        let (g, p) = ged_one("A /B/", vec![Fact::dated("BAPM", Some("1872"))]);
        assert_eq!(date_lines(&g, p.0), vec!["\u{2248} 1872".to_string()]);
    }

    #[test]
    fn burial_marks_dead_without_prefix() {
        let (g, p) = ged_one("A /B/", vec![Fact::dated("BURI", Some("1940"))]);
        assert!(is_dead(&g, p.0));
        assert_eq!(date_lines(&g, p.0), vec!["1940".to_string()]);
    }

    #[test]
    fn long_dates_split_lines() {
        let (g, p) = ged_one(
            "A /B/",
            vec![
                Fact::dated("BIRT", Some("12 MAY 1870")),
                Fact::dated("DEAT", Some("3 JUN 1941")),
            ],
        );
        assert_eq!(
            date_lines(&g, p.0),
            vec![
                "\u{2605} 12 MAY 1870".to_string(),
                "\u{271B} 3 JUN 1941".to_string()
            ]
        );
    }
}
