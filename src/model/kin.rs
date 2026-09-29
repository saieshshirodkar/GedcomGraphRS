use crate::model::gedcom::{FamilyId, GedcomData, PersonId};

pub fn count_ancestors(ged: &GedcomData, person: u32) -> u32 {
    let mut amount = 1u32;
    let mut stack: Vec<PersonId> = Vec::with_capacity(8);
    stack.push(PersonId(person));
    while let Some(cur) = stack.pop() {
        if amount > 100 {
            break;
        }
        if let Some(p) = ged.person(cur.0) {
            for fam in &p.parent_fams {
                if let Some(f) = ged.family(fam.0) {
                    for h in &f.husbands {
                        amount += 1;
                        stack.push(*h);
                    }
                    for w in &f.wives {
                        amount += 1;
                        stack.push(*w);
                    }
                }
            }
        }
    }
    amount
}

pub fn count_descendants(ged: &GedcomData, person: u32) -> u32 {
    let mut amount = 1u32;
    let mut stack: Vec<PersonId> = Vec::with_capacity(8);
    stack.push(PersonId(person));
    while let Some(cur) = stack.pop() {
        if amount > 100 {
            break;
        }
        if let Some(p) = ged.person(cur.0) {
            for fam in &p.spouse_fams {
                if let Some(f) = ged.family(fam.0) {
                    for c in &f.children {
                        amount += 1;
                        stack.push(*c);
                    }
                }
            }
        }
    }
    amount
}

pub fn parent_family_last(ged: &GedcomData, person: u32) -> Option<FamilyId> {
    ged.person(person)
        .and_then(|p| p.parent_fams.last().copied())
}

pub fn family_marriage_date(ged: &GedcomData, family: u32) -> Option<String> {
    ged.family(family).and_then(|f| {
        f.facts
            .iter()
            .find(|e| e.tag == "MARR")
            .and_then(|e| e.date.clone())
    })
}

pub fn spouses_of(ged: &GedcomData, family: u32, excluded: Option<PersonId>) -> Vec<PersonId> {
    let mut out: Vec<PersonId> = Vec::with_capacity(2);
    if let Some(f) = ged.family(family) {
        if let Some(h) = f.husbands.first() {
            out.push(*h);
        }
        if let Some(w) = f.wives.first() {
            out.push(*w);
        }
        for h in f.husbands.iter().skip(1) {
            out.push(*h);
        }
        for w in f.wives.iter().skip(1) {
            out.push(*w);
        }
    }
    if let Some(x) = excluded {
        out.retain(|p| *p != x);
    }
    if out.len() > 2 {
        out.truncate(2);
    }
    if excluded.is_some() && out.len() == 2 {
        out.truncate(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gedcom::{Fact, GFamily, GPerson, Name};

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
    fn ancestor_and_descendant_counts() {
        let g = sample();
        assert_eq!(count_ancestors(&g, 0), 1);
        assert_eq!(count_descendants(&g, 0), 1);
    }

    #[test]
    fn spouses_and_marriage() {
        let g = sample();
        assert_eq!(
            spouses_of(&g, 0, Option::None),
            vec![PersonId(0), PersonId(1)]
        );
        assert_eq!(spouses_of(&g, 0, Some(PersonId(0))), vec![PersonId(1)]);
        assert_eq!(family_marriage_date(&g, 0), Some("1895".to_string()));
        assert_eq!(family_marriage_date(&g, 9), Option::None);
        assert_eq!(parent_family_last(&g, 0), Option::None);
    }

    #[test]
    fn ancestor_count_caps() {
        let mut g = GedcomData::empty();
        let mut prev: Option<PersonId> = Option::None;
        for i in 0..105 {
            let p = g.push_person(GPerson {
                id: format!("I{i}"),
                names: vec![Name::of("P")],
                facts: Vec::new(),
                parent_fams: Vec::new(),
                spouse_fams: Vec::new(),
            });
            if let Some(par) = prev {
                let f = g.push_family(GFamily {
                    id: format!("F{i}"),
                    husbands: Vec::new(),
                    wives: Vec::new(),
                    children: Vec::new(),
                    facts: Vec::new(),
                });
                g.link_spouses(f, &[par], &[]);
                g.link_children(f, &[p]);
            }
            prev = Some(p);
        }
        g.reindex();
        assert_eq!(count_ancestors(&g, 104), 101);
        assert_eq!(count_descendants(&g, 0), 101);
    }

    #[test]
    fn spouses_truncate_to_two() {
        let mut g = GedcomData::empty();
        let mut ids: Vec<PersonId> = Vec::new();
        for id in ["I1", "I2", "I3"] {
            ids.push(g.push_person(GPerson {
                id: id.to_string(),
                names: vec![Name::of("P")],
                facts: Vec::new(),
                parent_fams: Vec::new(),
                spouse_fams: Vec::new(),
            }));
        }
        let f = g.push_family(GFamily {
            id: "F1".to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts: Vec::new(),
        });
        g.link_spouses(f, &[ids[0], ids[1]], &[ids[2]]);
        g.reindex();
        assert_eq!(spouses_of(&g, f.0, Option::None), vec![ids[0], ids[2]]);
    }
}
