use super::gedcom::{GFamily, GPerson, GedcomData};

impl GedcomData {
    pub fn empty() -> GedcomData {
        GedcomData::default()
    }

    pub fn reindex(&mut self) {
        let mut po: Vec<u32> = (0..self.persons.len() as u32).collect();
        po.sort_by(|a, b| {
            self.persons[*a as usize]
                .id
                .cmp(&self.persons[*b as usize].id)
        });
        self.person_order = po;
        let mut fo: Vec<u32> = (0..self.families.len() as u32).collect();
        fo.sort_by(|a, b| {
            self.families[*a as usize]
                .id
                .cmp(&self.families[*b as usize].id)
        });
        self.family_order = fo;
    }

    pub fn find_person(&self, id: &str) -> Option<u32> {
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
                return Some(idx as u32);
            } else if cur < id {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Option::None
    }

    pub fn find_family(&self, id: &str) -> Option<u32> {
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
                return Some(idx as u32);
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

    pub fn push_person(&mut self, person: GPerson) -> u32 {
        let idx = self.persons.len() as u32;
        self.persons.push(person);
        idx
    }

    pub fn push_family(&mut self, family: GFamily) -> u32 {
        let idx = self.families.len() as u32;
        self.families.push(family);
        idx
    }

    pub fn link_spouses(&mut self, family: u32, husbands: &[u32], wives: &[u32]) {
        for h in husbands {
            if let Some(p) = self.persons.get_mut(*h as usize) {
                if !p.spouse_fams.contains(&family) {
                    p.spouse_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(family as usize) {
                if !f.husbands.contains(h) {
                    f.husbands.push(*h);
                }
            }
        }
        for w in wives {
            if let Some(p) = self.persons.get_mut(*w as usize) {
                if !p.spouse_fams.contains(&family) {
                    p.spouse_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(family as usize) {
                if !f.wives.contains(w) {
                    f.wives.push(*w);
                }
            }
        }
    }

    pub fn link_children(&mut self, family: u32, children: &[u32]) {
        for c in children {
            if let Some(p) = self.persons.get_mut(*c as usize) {
                if !p.parent_fams.contains(&family) {
                    p.parent_fams.push(family);
                }
            }
            if let Some(f) = self.families.get_mut(family as usize) {
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
        assert_eq!(g.find_person("I1"), Some(1));
        assert_eq!(g.find_person("I2"), Some(0));
        assert_eq!(g.find_family("F1"), Some(0));
        assert_eq!(g.person_count(), 2);
        assert_eq!(g.family_count(), 1);
    }

    #[test]
    fn links_deduplicate() {
        let mut g = two();
        g.link_spouses(0, &[0], &[1]);
        g.link_spouses(0, &[0], &[1]);
        g.link_children(0, &[1]);
        g.link_children(0, &[1]);
        assert_eq!(g.families[0].husbands, vec![0]);
        assert_eq!(g.families[0].wives, vec![1]);
        assert_eq!(g.families[0].children, vec![1]);
        assert_eq!(g.persons[0].spouse_fams, vec![0]);
        assert_eq!(g.persons[1].parent_fams, vec![0]);
        assert_eq!(g.find_person("I1"), Some(1));
    }
}
