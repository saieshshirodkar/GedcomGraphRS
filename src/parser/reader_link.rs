use crate::model::gedcom::{Fact, FamilyId, GFamily, GPerson, GedcomData, Name, PersonId};
use crate::parser::reader::{build_arena, child_date, child_place, strip_ats};
use std::collections::HashMap;

pub fn parse_gedcom(text: &str) -> GedcomData {
    let arena = build_arena(text);
    let mut ged = GedcomData::empty();
    let mut person_ids: HashMap<String, PersonId> = HashMap::with_capacity(arena.roots.len());
    let mut family_ids: HashMap<String, FamilyId> = HashMap::with_capacity(arena.roots.len());
    for r in &arena.roots {
        let n = &arena.nodes[*r];
        if n.tag == "HEAD" || n.tag == "TRLR" {
            continue;
        }
        if n.tag == "INDI" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref).to_string();
            let idx = ged.push_person(GPerson {
                id: id.clone(),
                names: Vec::new(),
                facts: Vec::new(),
                parent_fams: Vec::new(),
                spouse_fams: Vec::new(),
            });
            person_ids.insert(id, idx);
        } else if n.tag == "FAM" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref).to_string();
            let idx = ged.push_family(GFamily {
                id: id.clone(),
                husbands: Vec::new(),
                wives: Vec::new(),
                children: Vec::new(),
                facts: Vec::new(),
            });
            family_ids.insert(id, idx);
        }
    }
    for r in &arena.roots {
        let n = &arena.nodes[*r];
        if n.tag == "INDI" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref);
            let Some(pi) = person_ids.get(id).copied() else {
                continue;
            };
            for c in &arena.kids[*r] {
                let kid = &arena.nodes[*c];
                match kid.tag.as_str() {
                    "NAME" => {
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.names.push(Name::of(&kid.value));
                        }
                    }
                    "SEX" => {
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.facts
                                .push(Fact::of("SEX", Some(kid.value.as_str()), Option::None));
                        }
                    }
                    "FAMC" => {
                        let fid = strip_ats(&kid.value);
                        if let Some(fi) = family_ids.get(fid).copied() {
                            if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                                if !p.parent_fams.contains(&fi) {
                                    p.parent_fams.push(fi);
                                }
                            }
                            if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                                if !f.children.contains(&pi) {
                                    f.children.push(pi);
                                }
                            }
                        }
                    }
                    "FAMS" => {
                        let fid = strip_ats(&kid.value);
                        if let Some(fi) = family_ids.get(fid).copied() {
                            if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                                if !p.spouse_fams.contains(&fi) {
                                    p.spouse_fams.push(fi);
                                }
                            }
                        }
                    }
                    _ => {
                        let date = child_date(&arena, *c);
                        let place = child_place(&arena, *c);
                        let val = if kid.value.is_empty() {
                            Option::None
                        } else {
                            Some(kid.value.as_str())
                        };
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.facts.push(Fact {
                                tag: kid.tag.clone(),
                                value: val.map(str::to_string),
                                date,
                                place,
                            });
                        }
                    }
                }
            }
        } else if n.tag == "FAM" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref);
            let Some(fi) = family_ids.get(id).copied() else {
                continue;
            };
            for c in &arena.kids[*r] {
                let kid = &arena.nodes[*c];
                match kid.tag.as_str() {
                    "HUSB" => {
                        let pid = strip_ats(&kid.value);
                        if let Some(pi) = person_ids.get(pid).copied() {
                            if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                                if !f.husbands.contains(&pi) {
                                    f.husbands.push(pi);
                                }
                            }
                            if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                                if !p.spouse_fams.contains(&fi) {
                                    p.spouse_fams.push(fi);
                                }
                            }
                        }
                    }
                    "WIFE" => {
                        let pid = strip_ats(&kid.value);
                        if let Some(pi) = person_ids.get(pid).copied() {
                            if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                                if !f.wives.contains(&pi) {
                                    f.wives.push(pi);
                                }
                            }
                            if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                                if !p.spouse_fams.contains(&fi) {
                                    p.spouse_fams.push(fi);
                                }
                            }
                        }
                    }
                    "CHIL" => {
                        let pid = strip_ats(&kid.value);
                        if let Some(pi) = person_ids.get(pid).copied() {
                            if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                                if !f.children.contains(&pi) {
                                    f.children.push(pi);
                                }
                            }
                            if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                                if !p.parent_fams.contains(&fi) {
                                    p.parent_fams.push(fi);
                                }
                            }
                        }
                    }
                    _ => {
                        let date = child_date(&arena, *c);
                        let place = child_place(&arena, *c);
                        if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                            f.facts.push(Fact {
                                tag: kid.tag.clone(),
                                value: Option::None,
                                date,
                                place,
                            });
                        }
                    }
                }
            }
        }
    }
    ged.reindex();
    ged
}
