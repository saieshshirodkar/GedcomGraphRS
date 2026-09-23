use crate::model::gedcom::GedcomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Gender {
    #[default]
    None = 0,
    Male = 1,
    Female = 2,
    Undefined = 3,
    Other = 4,
}

impl Gender {
    pub fn from_sex_value(value: Option<&str>) -> Gender {
        match value {
            Some("M") => Gender::Male,
            Some("F") => Gender::Female,
            Some("U") => Gender::Undefined,
            Some(_) => Gender::Other,
            Option::None => Gender::None,
        }
    }

    pub fn of_person(ged: &GedcomData, person: u32) -> Gender {
        let mut out = Gender::None;
        if let Some(p) = ged.person(person) {
            for f in &p.facts {
                if f.tag == "SEX" {
                    out = Gender::from_sex_value(f.value.as_deref());
                    break;
                }
            }
        }
        out
    }

    pub fn is_male(self) -> bool {
        self == Gender::Male
    }

    pub fn is_female(self) -> bool {
        self == Gender::Female
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GPerson, GedcomData, Name};

    #[test]
    fn sex_mapping() {
        assert_eq!(Gender::from_sex_value(Some("M")), Gender::Male);
        assert_eq!(Gender::from_sex_value(Some("F")), Gender::Female);
        assert_eq!(Gender::from_sex_value(Some("U")), Gender::Undefined);
        assert_eq!(Gender::from_sex_value(Some("X")), Gender::Other);
        assert_eq!(Gender::from_sex_value(Option::None), Gender::None);
    }

    #[test]
    fn male_female_helpers() {
        assert!(Gender::Male.is_male());
        assert!(!Gender::Female.is_male());
        assert!(Gender::Female.is_female());
        assert!(!Gender::Male.is_female());
    }

    #[test]
    fn of_person_reads_sex_fact() {
        let mut ged = GedcomData::empty();
        ged.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("A")],
            facts: vec![Fact::of("SEX", Some("F"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        ged.reindex();
        assert_eq!(Gender::of_person(&ged, 0), Gender::Female);
        assert_eq!(Gender::of_person(&ged, 9), Gender::None);
    }
}
