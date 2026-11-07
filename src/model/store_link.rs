use super::gedcom::{FamilyId, GedcomData, PersonId};
use crate::config::Gender;
use crate::model::helpers::person_gender;

impl GedcomData {
    pub fn link_spouses(&mut self, family: FamilyId, husbands: &[PersonId], wives: &[PersonId]) {
        for h in husbands {
            if let Some(p) = self.persons.get_mut(usize::from(*h))
                && !p.spouse_fams.contains(&family)
            {
                p.spouse_fams.push(family);
            }
            if let Some(f) = self.families.get_mut(usize::from(family))
                && !f.husbands.contains(h)
            {
                f.husbands.push(*h);
            }
        }
        for w in wives {
            if let Some(p) = self.persons.get_mut(usize::from(*w))
                && !p.spouse_fams.contains(&family)
            {
                p.spouse_fams.push(family);
            }
            if let Some(f) = self.families.get_mut(usize::from(family))
                && !f.wives.contains(w)
            {
                f.wives.push(*w);
            }
        }
    }

    pub fn link_spouse(&mut self, family: FamilyId, person: PersonId) {
        self.link_spouse_person(family, person);
        let female = person_gender(self, person.0) == Gender::Female;
        if let Some(f) = self.families.get_mut(usize::from(family)) {
            if f.husbands.contains(&person) || f.wives.contains(&person) {
                return;
            }
            if female {
                f.wives.push(person);
            } else {
                f.husbands.push(person);
            }
        }
    }

    pub fn link_spouse_person(&mut self, family: FamilyId, person: PersonId) {
        if let Some(p) = self.persons.get_mut(usize::from(person))
            && !p.spouse_fams.contains(&family)
        {
            p.spouse_fams.push(family);
        }
    }

    pub fn link_child(&mut self, family: FamilyId, child: PersonId) {
        if let Some(p) = self.persons.get_mut(usize::from(child))
            && !p.parent_fams.contains(&family)
        {
            p.parent_fams.push(family);
        }
        if let Some(f) = self.families.get_mut(usize::from(family))
            && !f.children.contains(&child)
        {
            f.children.push(child);
        }
    }

    pub fn link_children(&mut self, family: FamilyId, children: &[PersonId]) {
        for c in children {
            if let Some(p) = self.persons.get_mut(usize::from(*c))
                && !p.parent_fams.contains(&family)
            {
                p.parent_fams.push(family);
            }
            if let Some(f) = self.families.get_mut(usize::from(family))
                && !f.children.contains(c)
            {
                f.children.push(*c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GFamily, GPerson, Name};

    fn two() -> GedcomData {
        let mut g = GedcomData::empty();
        g.push_person(GPerson {
            id: "I2".to_string(),
            names: vec![Name::of("B")],
            facts: Vec::new(),
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
            ..GPerson::default()
        });
        g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("A")],
            facts: vec![Fact::of("SEX", Some("M"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
            ..GPerson::default()
        });
        g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
            ..GFamily::default()
        });
        g
    }

    #[test]
    fn links_deduplicate() {
        let mut g = two();
        g.link_spouses(FamilyId(0), &[PersonId(0)], &[PersonId(1)]);
        g.link_spouses(FamilyId(0), &[PersonId(0)], &[PersonId(1)]);
        g.link_children(FamilyId(0), &[PersonId(1)]);
        g.link_children(FamilyId(0), &[PersonId(1)]);
        g.reindex();
        assert_eq!(g.families[0].husbands, vec![PersonId(0)]);
        assert_eq!(g.families[0].wives, vec![PersonId(1)]);
        assert_eq!(g.families[0].children, vec![PersonId(1)]);
        assert_eq!(g.persons[0].spouse_fams, vec![FamilyId(0)]);
        assert_eq!(g.persons[1].parent_fams, vec![FamilyId(0)]);
        assert_eq!(g.find_person("I1"), Some(PersonId(1)));
    }

    #[test]
    fn link_spouse_fills_family_side() {
        let mut g = two();
        g.link_spouse(FamilyId(0), PersonId(1));
        assert_eq!(g.families[0].husbands, vec![PersonId(1)]);
        assert_eq!(g.persons[1].spouse_fams, vec![FamilyId(0)]);
        g.link_spouse(FamilyId(0), PersonId(1));
        assert_eq!(g.families[0].husbands, vec![PersonId(1)]);
        g.link_spouse(FamilyId(0), PersonId(0));
        assert_eq!(g.families[0].husbands, vec![PersonId(1), PersonId(0)]);
    }

    #[test]
    fn link_spouse_female_goes_to_wives() {
        let mut g = two();
        if let Some(p) = g.persons.get_mut(0) {
            p.facts.push(Fact::of("SEX", Some("F"), Option::None));
        }
        g.link_spouse(FamilyId(0), PersonId(0));
        assert_eq!(g.families[0].wives, vec![PersonId(0)]);
        assert!(g.families[0].husbands.is_empty());
    }

    fn child(id: &str, name: &str, birth: Option<&str>) -> GPerson {
        let mut facts: Vec<Fact> = Vec::new();
        if let Some(d) = birth {
            facts.push(Fact::dated("BIRT", Some(d)));
        }
        GPerson {
            id: id.to_string(),
            names: vec![Name::of(name)],
            facts,
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
            ..GPerson::default()
        }
    }

    #[test]
    fn children_oldest_first() {
        let mut g = GedcomData::empty();
        let c = g.push_person(child("C", "Cid", Some("1950")));
        let a = g.push_person(child("A", "Al", Some("1920")));
        let b = g.push_person(child("B", "Bo", Some("1980")));
        let f = g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
            ..GFamily::default()
        });
        g.link_children(f, &[c, a, b]);
        g.reindex();
        assert_eq!(g.families[0].children, vec![a, c, b]);
    }

    #[test]
    fn children_without_birth_last() {
        let mut g = GedcomData::empty();
        let u = g.push_person(child("U", "Zed", Option::None));
        let d = g.push_person(child("D", "Al", Some("1920")));
        let f = g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
            ..GFamily::default()
        });
        g.link_children(f, &[u, d]);
        g.reindex();
        assert_eq!(g.families[0].children, vec![d, u]);
    }

    #[test]
    fn children_without_birth_alphabetical() {
        let mut g = GedcomData::empty();
        let z = g.push_person(child("Z", "Zed", Option::None));
        let m = g.push_person(child("M", "Max", Option::None));
        let a = g.push_person(child("A", "Al", Option::None));
        let f = g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
            ..GFamily::default()
        });
        g.link_children(f, &[z, m, a]);
        g.reindex();
        assert_eq!(g.families[0].children, vec![a, m, z]);
    }
}
