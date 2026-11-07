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
    use crate::model::gedcom::Name;

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
}
