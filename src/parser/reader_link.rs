use crate::model::gedcom::{
    Dialect, Fact, FamilyId, GFamily, GPerson, GedcomData, Name, NamePiece, PersonId,
};
use crate::parser::reader::{
    build_arena, child_date, child_place, child_text, resolve_pointer, strip_ats,
};
use std::collections::HashMap;

fn read_version(arena: &crate::parser::reader::Arena) -> (String, Dialect) {
    for r in &arena.roots {
        if arena.nodes[*r].tag != "HEAD" {
            continue;
        }
        for c in &arena.kids[*r] {
            if arena.nodes[*c].tag != "GEDC" {
                continue;
            }
            if let Some(v) = child_text(arena, *c, "VERS") {
                let major = v.split('.').next().unwrap_or_default();
                let dialect = if major.starts_with('7') {
                    Dialect::Seven
                } else if major.starts_with('5') {
                    Dialect::FiveFive
                } else {
                    Dialect::Unknown
                };
                return (v, dialect);
            }
        }
    }
    (String::new(), Dialect::Unknown)
}

pub fn parse_gedcom(text: &str) -> GedcomData {
    let arena = build_arena(text);
    let mut ged = GedcomData::empty();
    let (version, dialect) = read_version(&arena);
    ged.version = version;
    ged.dialect = dialect;
    let mut person_ids: HashMap<String, PersonId> = HashMap::with_capacity(arena.roots.len());
    let mut family_ids: HashMap<String, FamilyId> = HashMap::with_capacity(arena.roots.len());
    let mut shadowed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for r in &arena.roots {
        let n = &arena.nodes[*r];
        if n.tag == "HEAD" || n.tag == "TRLR" {
            continue;
        }
        if n.tag == "INDI" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref).to_string();
            if person_ids.contains_key(&id) {
                shadowed.insert(*r);
                continue;
            }
            let idx = ged.push_person(GPerson {
                id: id.clone(),
                ..GPerson::default()
            });
            person_ids.insert(id, idx);
        } else if n.tag == "FAM" && !n.xref.is_empty() {
            let id = strip_ats(&n.xref).to_string();
            if family_ids.contains_key(&id) {
                shadowed.insert(*r);
                continue;
            }
            let idx = ged.push_family(GFamily {
                id: id.clone(),
                ..GFamily::default()
            });
            family_ids.insert(id, idx);
        }
    }
    for r in &arena.roots {
        let n = &arena.nodes[*r];
        if n.tag == "INDI" && !n.xref.is_empty() {
            if shadowed.contains(r) {
                continue;
            }
            let id = strip_ats(&n.xref);
            let Some(pi) = person_ids.get(id).copied() else {
                continue;
            };
            for c in &arena.kids[*r] {
                let kid = &arena.nodes[*c];
                match kid.tag.as_str() {
                    "NAME" => {
                        let pieces: Vec<NamePiece> = arena.kids[*c]
                            .iter()
                            .filter_map(|gc| arena.nodes.get(*gc))
                            .filter(|gc| {
                                matches!(
                                    gc.tag.as_str(),
                                    "GIVN" | "SURN" | "NICK" | "NPFX" | "NSFX" | "SPFX"
                                ) && !gc.value.is_empty()
                            })
                            .map(|gc| NamePiece {
                                tag: gc.tag.clone(),
                                value: gc.value.clone(),
                            })
                            .collect();
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.names.push(Name::of(&kid.value));
                            p.pieces.extend(pieces);
                        }
                    }
                    "GIVN" | "SURN" | "NICK" | "NPFX" | "NSFX" | "SPFX" => {
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.pieces.push(NamePiece {
                                tag: kid.tag.clone(),
                                value: kid.value.clone(),
                            });
                        }
                    }
                    "RESN" => {
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.restricted = kid.value.split_whitespace().any(|v| {
                                v.eq_ignore_ascii_case("confidential")
                                    || v.eq_ignore_ascii_case("privacy")
                            });
                        }
                    }
                    "SEX" => {
                        if let Some(p) = ged.persons.get_mut(usize::from(pi)) {
                            p.facts
                                .push(Fact::of("SEX", Some(kid.value.as_str()), Option::None));
                        }
                    }
                    "FAMC" => {
                        let Some(fid) = resolve_pointer(&kid.value) else {
                            continue;
                        };
                        if let Some(fi) = family_ids.get(fid).copied() {
                            ged.link_child(fi, pi);
                        }
                    }
                    "FAMS" => {
                        let Some(fid) = resolve_pointer(&kid.value) else {
                            continue;
                        };
                        if let Some(fi) = family_ids.get(fid).copied() {
                            ged.link_spouse_person(fi, pi);
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
            if shadowed.contains(r) {
                continue;
            }
            let id = strip_ats(&n.xref);
            let Some(fi) = family_ids.get(id).copied() else {
                continue;
            };
            for c in &arena.kids[*r] {
                let kid = &arena.nodes[*c];
                match kid.tag.as_str() {
                    "HUSB" => {
                        let Some(pid) = resolve_pointer(&kid.value) else {
                            continue;
                        };
                        if let Some(pi) = person_ids.get(pid).copied() {
                            if let Some(f) = ged.families.get_mut(usize::from(fi))
                                && !f.husbands.contains(&pi)
                            {
                                f.husbands.push(pi);
                            }
                            ged.link_spouse_person(fi, pi);
                        }
                    }
                    "WIFE" => {
                        let Some(pid) = resolve_pointer(&kid.value) else {
                            continue;
                        };
                        if let Some(pi) = person_ids.get(pid).copied() {
                            if let Some(f) = ged.families.get_mut(usize::from(fi))
                                && !f.wives.contains(&pi)
                            {
                                f.wives.push(pi);
                            }
                            ged.link_spouse_person(fi, pi);
                        }
                    }
                    "CHIL" => {
                        let Some(pid) = resolve_pointer(&kid.value) else {
                            continue;
                        };
                        if let Some(pi) = person_ids.get(pid).copied() {
                            ged.link_child(fi, pi);
                        }
                    }
                    "RESN" => {
                        if let Some(f) = ged.families.get_mut(usize::from(fi)) {
                            f.restricted = kid.value.split_whitespace().any(|v| {
                                v.eq_ignore_ascii_case("confidential")
                                    || v.eq_ignore_ascii_case("privacy")
                            });
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
    for i in 0..ged.persons.len() {
        let pid = PersonId(u32::try_from(i).unwrap_or(u32::MAX));
        let fams = ged.persons[i].spouse_fams.clone();
        for fam in fams {
            ged.link_spouse(fam, pid);
        }
    }
    ged.reindex();
    ged
}
