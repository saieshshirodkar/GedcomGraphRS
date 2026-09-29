use super::gedcom::{FamilyId, GFamily, GPerson, GedcomData, PersonId};

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
    }

    pub fn find_person(&self, id: &str) -> Option<PersonId> {
        let mut lo = 0usize;
        let mut hi = self.person_order.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let idx = self.person_order[mid] as usize;
            let cur = self
                .persons
                .get(idx)
                .map(|p| p.id.as_str())
                .unwrap_or_default();
            if cur == id {
                return u32::try_from(idx).ok().map(PersonId);
            } else if cur < id {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Option::None
    }

    pub fn find_family(&self, id: &str) -> Option<FamilyId> {
        let mut lo = 0usize;
        let mut hi = self.family_order.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let idx = self.family_order[mid] as usize;
            let cur = self
                .families
                .get(idx)
                .map(|f| f.id.as_str())
                .unwrap_or_default();
            if cur == id {
                return u32::try_from(idx).ok().map(FamilyId);
            } else if cur < id {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Option::None
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
        idx
    }

    pub fn push_family(&mut self, family: GFamily) -> FamilyId {
        let idx = FamilyId(u32::try_from(self.families.len()).unwrap_or(u32::MAX));
        self.families.push(family);
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
        self.reindex();
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
        });
        g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("A")],
            facts: Vec::new(),
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
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
        assert_eq!(g.families[0].husbands, vec![PersonId(0)]);
        assert_eq!(g.families[0].wives, vec![PersonId(1)]);
        assert_eq!(g.families[0].children, vec![PersonId(1)]);
        assert_eq!(g.persons[0].spouse_fams, vec![FamilyId(0)]);
        assert_eq!(g.persons[1].parent_fams, vec![FamilyId(0)]);
        assert_eq!(g.find_person("I1"), Some(PersonId(1)));
    }
}
