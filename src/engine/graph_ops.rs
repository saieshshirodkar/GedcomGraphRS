use crate::engine::graph::Graph;
use crate::lines::duplicate::DupLine;
use crate::model::gedcom::PersonId;
use crate::model::kin::spouses_of;

impl Graph {
    pub fn spouses(&self, family: u32, excluded: Option<u32>) -> Vec<PersonId> {
        spouses_of(&self.gedcom, family, excluded.map(PersonId))
    }

    pub fn are_siblings(&self, first: Option<u32>, second: Option<u32>) -> bool {
        if let (Some(a), Some(b)) = (first, second) {
            let fa = self
                .gedcom
                .person(a)
                .map(|p| p.parent_fams.clone())
                .unwrap_or_default();
            let fb = self
                .gedcom
                .person(b)
                .map(|p| p.parent_fams.clone())
                .unwrap_or_default();
            if let (Some(x), Some(y)) = (fa.last(), fb.last()) {
                return x == y && !fa.is_empty() && !fb.is_empty();
            }
        }
        false
    }

    pub(crate) fn check_for_duplicate(&mut self, pid: u32, spouse_family: Option<u32>) {
        let (mini, person, generation) = self
            .anim
            .persons
            .get(pid as usize)
            .map(|p| (p.base.mini, p.person, p.base.generation))
            .unwrap_or((true, u32::MAX, 0));
        if mini {
            return;
        }
        let existing: Vec<u32> = self.registered_persons();
        for q in existing {
            let (omini, operson, ogen) = self
                .anim
                .persons
                .get(q as usize)
                .map(|p| (p.base.mini, p.person, p.base.generation))
                .unwrap_or((true, u32::MAX, 0));
            if omini {
                continue;
            }
            if operson == person {
                let same_family = match (self.old_family_of(q), spouse_family) {
                    (_, Option::None) => true,
                    (Option::None, _) => true,
                    (Some(a), Some(b)) => a == b,
                };
                if same_family {
                    if let Some(p) = self.anim.persons.get_mut(pid as usize) {
                        p.duplicate = true;
                    }
                }
                if self.with_duplicate_lines {
                    let line = DupLine::of(&self.gedcom, q, pid, &self.anim.persons);
                    self.anim.duplicate_lines.push(line);
                }
            } else if ogen == generation {
                if let (Some(a), Some(b)) = (self.old_family_of(q), spouse_family) {
                    if a == b {
                        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
                            p.duplicate = true;
                        }
                    }
                }
            }
        }
    }

    fn registered_persons(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for nid in self.anim.nodes.clone() {
            out.extend(self.anim.node_persons(nid));
        }
        out
    }

    fn old_family_of(&self, pid: u32) -> Option<u32> {
        self.anim.persons.get(pid as usize).and_then(|p| {
            if let Some(f) = p.family {
                self.anim
                    .families
                    .get(f as usize)
                    .map(|fam| fam.spouse_family)
            } else {
                Option::None
            }
        })
    }
}
