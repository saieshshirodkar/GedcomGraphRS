use crate::config::Gender;
use crate::model::gedcom::GedcomData;

pub fn person_is_dead(ged: &GedcomData, person: u32) -> bool {
    if let Some(p) = ged.person(person) {
        for f in &p.facts {
            if f.tag == "DEAT" || f.tag == "BURI" {
                return true;
            }
        }
    }
    false
}

pub fn person_gender(ged: &GedcomData, person: u32) -> Gender {
    Gender::of_person(ged, person)
}

pub fn person_is_male(ged: &GedcomData, person: u32) -> bool {
    person_gender(ged, person).is_male()
}

pub fn person_is_female(ged: &GedcomData, person: u32) -> bool {
    person_gender(ged, person).is_female()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GFamily, GPerson, Name, essence};

    fn sample() -> GedcomData {
        let mut g = GedcomData::empty();
        let a = g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("John /Smith/")],
            facts: vec![Fact::of("SEX", Some("M"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        let b = g.push_person(GPerson {
            id: "I2".to_string(),
            names: vec![Name::of("Mary /Doe/")],
            facts: vec![Fact::of("SEX", Some("F"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        let f = g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: vec![Fact::dated("MARR", Some("1895"))],
        });
        g.link_spouses(f, &[a], &[b]);
        g.reindex();
        g
    }

    #[test]
    fn dead_detection() {
        let mut g = sample();
        assert!(!person_is_dead(&g, 0));
        if let Some(p) = g.persons.get_mut(0) {
            p.facts.push(Fact::dated("DEAT", Some("1940")));
        }
        assert!(person_is_dead(&g, 0));
        assert!(!person_is_dead(&g, 99));
    }

    #[test]
    fn gender_helpers() {
        let g = sample();
        assert!(person_is_male(&g, 0));
        assert!(!person_is_female(&g, 0));
        assert!(person_is_female(&g, 1));
        assert_eq!(person_gender(&g, 1), Gender::Female);
    }

    #[test]
    fn essence_strips_slashes() {
        let g = sample();
        assert_eq!(essence(&g, Some(0)), "John Smith");
        assert_eq!(essence(&g, Some(1)), "Mary Doe");
        assert_eq!(essence(&g, Option::None), "[No name]");
        assert_eq!(essence(&g, Some(77)), "[No name]");
    }
}
