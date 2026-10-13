use super::gedcom::{FamilyId, GFamily, GPerson, GedcomData, PersonId};
use crate::parser::date::estimate_year;

impl GedcomData {
    pub fn empty() -> GedcomData {
        GedcomData::default()
    }

    pub fn reindex(&mut self) {
        let np = u32::try_from(self.persons.len()).unwrap_or(u32::MAX);
        let mut po: Vec<u32> = (0..np).collect();
        po.sort_by(|a, b| {
            self.persons[*a as usize]
                .id
                .cmp(&self.persons[*b as usize].id)
        });
        self.person_order = po;
        let nf = u32::try_from(self.families.len()).unwrap_or(u32::MAX);
        let mut fo: Vec<u32> = (0..nf).collect();
        fo.sort_by(|a, b| {
            self.families[*a as usize]
                .id
                .cmp(&self.families[*b as usize].id)
        });
        self.family_order = fo;
        self.sort_children();
    }

    pub fn sort_children(&mut self) {
        for i in 0..self.families.len() {
            let mut kids = std::mem::take(&mut self.families[i].children);
            kids.sort_by_key(|a| kid_key(self, *a));
            self.families[i].children = kids;
        }
    }

    pub fn find_person(&self, id: &str) -> Option<PersonId> {
        let idx = search(&self.person_order, id, |i| {
            Some(self.persons.get(i)?.id.as_str())
        })?;
        u32::try_from(idx).ok().map(PersonId)
    }

    pub fn find_family(&self, id: &str) -> Option<FamilyId> {
        let idx = search(&self.family_order, id, |i| {
            Some(self.families.get(i)?.id.as_str())
        })?;
        u32::try_from(idx).ok().map(FamilyId)
    }

    pub fn person(&self, idx: u32) -> Option<&GPerson> {
        self.persons.get(idx as usize)
    }

    pub fn family(&self, idx: u32) -> Option<&GFamily> {
        self.families.get(idx as usize)
    }

    pub fn person_count(&self) -> usize {
        self.persons.len()
    }

    pub fn family_count(&self) -> usize {
        self.families.len()
    }

    pub fn push_person(&mut self, person: GPerson) -> PersonId {
        let idx = PersonId(u32::try_from(self.persons.len()).unwrap_or(u32::MAX));
        self.persons.push(person);
        self.person_order.clear();
        idx
    }

    pub fn push_family(&mut self, family: GFamily) -> FamilyId {
        let idx = FamilyId(u32::try_from(self.families.len()).unwrap_or(u32::MAX));
        self.families.push(family);
        self.family_order.clear();
        idx
    }

    pub fn link_spouses(&mut self, family: FamilyId, husbands: &[PersonId], wives: &[PersonId]) {
        for h in husbands {
            if let Some(p) = self.persons.get_mut(usize::from(*h)) {
                if !p.spouse_fams.contains(&family) {
                    p.spouse_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(usize::from(family)) {
                if !f.husbands.contains(h) {
                    f.husbands.push(*h);
                }
            }
        }
        for w in wives {
            if let Some(p) = self.persons.get_mut(usize::from(*w)) {
                if !p.spouse_fams.contains(&family) {
                    p.spouse_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(usize::from(family)) {
                if !f.wives.contains(w) {
                    f.wives.push(*w);
                }
            }
        }
    }

    pub fn link_spouse(&mut self, family: FamilyId, person: PersonId) {
        if let Some(p) = self.persons.get_mut(usize::from(person)) {
            if !p.spouse_fams.contains(&family) {
                p.spouse_fams.push(family);
            }
        }
    }

    pub fn link_child(&mut self, family: FamilyId, child: PersonId) {
        if let Some(p) = self.persons.get_mut(usize::from(child)) {
            if !p.parent_fams.contains(&family) {
                p.parent_fams.push(family);
            }
        }
        if let Some(f) = self.families.get_mut(usize::from(family)) {
            if !f.children.contains(&child) {
                f.children.push(child);
            }
        }
    }

    pub fn link_children(&mut self, family: FamilyId, children: &[PersonId]) {
        for c in children {
            if let Some(p) = self.persons.get_mut(usize::from(*c)) {
                if !p.parent_fams.contains(&family) {
                    p.parent_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(usize::from(family)) {
                if !f.children.contains(c) {
                    f.children.push(*c);
                }
            }
        }
    }
}

fn search<'a>(order: &[u32], id: &str, key: impl Fn(usize) -> Option<&'a str>) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = order.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let idx = order[mid] as usize;
        let cur = key(idx)?;
        if cur == id {
            return Some(idx);
        } else if cur < id {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    Option::None
}

fn birth_year(p: &GPerson) -> Option<i32> {
    p.facts
        .iter()
        .find(|f| f.tag == "BIRT" && f.date.is_some())
        .and_then(|f| f.date.as_deref())
        .and_then(estimate_year)
}

fn kid_key(ged: &GedcomData, person: PersonId) -> (bool, i32, &str) {
    let name = ged
        .person(person.0)
        .and_then(|p| p.names.first())
        .map(|n| n.display.as_str());
    let name = name.unwrap_or("");
    match ged.persons.get(usize::from(person)).and_then(birth_year) {
        Some(y) => (false, y, name),
        Option::None => (true, i32::MAX, name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GPerson, Name};

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
            facts: Vec::new(),
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
    fn miss_before_index() {
        let g = GedcomData::empty();
        assert_eq!(g.find_person("I1"), Option::None);
        assert_eq!(g.find_family("F1"), Option::None);
        assert!(g.person(99).is_none());
        assert!(g.family(99).is_none());
        assert_eq!(g.person_count(), 0);
        assert_eq!(g.family_count(), 0);
    }

    #[test]
    fn reindex_sorts_lookup() {
        let mut g = two();
        g.reindex();
        assert_eq!(g.find_person("I1"), Some(PersonId(1)));
        assert_eq!(g.find_person("I2"), Some(PersonId(0)));
        assert_eq!(g.find_family("F1"), Some(FamilyId(0)));
        assert_eq!(g.person_count(), 2);
        assert_eq!(g.family_count(), 1);
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
