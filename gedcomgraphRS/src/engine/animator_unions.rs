use crate::engine::animator::Animator;
use crate::layout::union::Union;

impl Animator {
    pub(crate) fn build_unions(&mut self) {
        for r in 0..self.group_rows.len() {
            let groups = self
                .group_rows
                .get(r)
                .map(|x| x.groups.clone())
                .unwrap_or_default();
            for gid in groups {
                let generation = self
                    .groups
                    .get(gid as usize)
                    .map(|g| g.generation)
                    .unwrap_or(0);
                let members = self
                    .groups
                    .get(gid as usize)
                    .map(|g| g.list.clone())
                    .unwrap_or_default();
                let row_unions = self
                    .union_rows
                    .get(r)
                    .map(|x| x.unions.clone())
                    .unwrap_or_default();
                let mut uid: Option<u32> = Option::None;
                let mut join = false;
                for nid in &members {
                    if self.node_is_ancestor(*nid) {
                        for u in &row_unions {
                            let anc = self.unions.get(*u as usize).and_then(|x| x.ancestor);
                            if anc == Some(*nid) {
                                uid = Some(*u);
                                join = true;
                                break;
                            }
                        }
                        if uid.is_none() {
                            let id = self.unions.len() as u32;
                            let mut nu = Union::of(self.node_generation(*nid));
                            nu.ancestor = Some(*nid);
                            self.unions.push(nu);
                            uid = Some(id);
                        }
                        break;
                    }
                }
                let target = if let Some(u) = uid {
                    u
                } else {
                    let id = self.unions.len() as u32;
                    self.unions.push(Union::of(generation));
                    id
                };
                if join {
                    let anc = self.unions.get(target as usize).and_then(|u| u.ancestor);
                    for nid in &members {
                        if Some(*nid) != anc {
                            if let Some(u) = self.unions.get_mut(target as usize) {
                                u.list.push(*nid);
                            }
                        }
                    }
                } else {
                    for nid in &members {
                        if let Some(u) = self.unions.get_mut(target as usize) {
                            u.list.push(*nid);
                        }
                    }
                    if let Some(row) = self.union_rows.get_mut(r) {
                        row.unions.push(target);
                    }
                }
                let done = self
                    .unions
                    .get(target as usize)
                    .map(|u| u.list.clone())
                    .unwrap_or_default();
                for nid in done {
                    self.set_node_union(nid, Some(target));
                    if let Some(m) = self.node_main_person(nid) {
                        if self.persons.get(m as usize).is_some_and(|p| p.is_fulcrum()) {
                            if let Some(u) = self.unions.get_mut(target as usize) {
                                u.ancestor = Some(nid);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn link_union_row(&mut self, row: usize) {
        let unions = self
            .union_rows
            .get(row)
            .map(|r| r.unions.clone())
            .unwrap_or_default();
        let mut previous: Option<u32> = Option::None;
        for uid in unions {
            if let Some(u) = self.unions.get_mut(uid as usize) {
                u.prev = previous;
            }
            if let Some(p) = previous {
                if let Some(u) = self.unions.get_mut(p as usize) {
                    u.next = Some(uid);
                }
            }
            previous = Some(uid);
        }
    }
}
