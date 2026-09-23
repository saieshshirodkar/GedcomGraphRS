use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn node_persons(&self, nid: NodeId) -> Vec<u32> {
        match nid {
            NodeId::Person(i) => {
                if self.persons.get(i as usize).is_some() {
                    vec![i]
                } else {
                    Vec::new()
                }
            }
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .map(|f| f.partners.clone())
                .unwrap_or_default(),
        }
    }

    pub(crate) fn node_person_count(&self, nid: NodeId) -> usize {
        match nid {
            NodeId::Person(i) => {
                if self.persons.get(i as usize).is_some() {
                    1
                } else {
                    0
                }
            }
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .map(|f| f.partners.len())
                .unwrap_or(0),
        }
    }

    pub(crate) fn node_main_person(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => {
                if self
                    .persons
                    .get(i as usize)
                    .is_some_and(|p| !p.half_sibling)
                {
                    Some(i)
                } else {
                    Option::None
                }
            }
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| {
                f.partners
                    .iter()
                    .find(|p| !self.person_acquired(**p))
                    .copied()
            }),
        }
    }

    pub(crate) fn node_main_origin(&self, nid: NodeId) -> Option<NodeId> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.origin),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| {
                self.node_main_person(nid)
                    .and_then(|m| self.person_origin(m, f))
            }),
        }
    }

    fn person_origin(
        &self,
        pid: u32,
        _fam: &crate::nodes::family::FamilyNodeData,
    ) -> Option<NodeId> {
        self.persons.get(pid as usize).and_then(|p| p.origin)
    }

    pub(crate) fn node_partner_origin(&self, nid: NodeId, slot: usize) -> Option<NodeId> {
        match nid {
            NodeId::Person(_) => self.node_main_origin(nid),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| {
                f.partners
                    .get(slot)
                    .copied()
                    .and_then(|p| self.persons.get(p as usize).and_then(|q| q.origin))
            }),
        }
    }

    pub(crate) fn node_husband(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get(i as usize) {
                    if let Some(f) = p.family {
                        return self
                            .families
                            .get(f as usize)
                            .and_then(|fam| fam.partners.first().copied());
                    }
                }
                Some(i)
            }
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .and_then(|f| f.partners.first().copied()),
        }
    }

    pub(crate) fn node_wife(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get(i as usize) {
                    if let Some(f) = p.family {
                        return self.families.get(f as usize).and_then(|fam| {
                            if fam.partners.len() > 1 {
                                fam.partners.get(1).copied()
                            } else {
                                fam.partners.first().copied()
                            }
                        });
                    }
                }
                Some(i)
            }
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| {
                if f.partners.len() > 1 {
                    f.partners.get(1).copied()
                } else {
                    f.partners.first().copied()
                }
            }),
        }
    }

    pub(crate) fn node_partner(&self, nid: NodeId, slot: usize) -> Option<u32> {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get(i as usize) {
                    if let Some(f) = p.family {
                        return self
                            .families
                            .get(f as usize)
                            .and_then(|fam| fam.partners.get(slot).copied());
                    }
                    if slot == 0 {
                        return Some(i);
                    }
                }
                Option::None
            }
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .and_then(|f| f.partners.get(slot).copied()),
        }
    }

    pub(crate) fn person_acquired(&self, pid: u32) -> bool {
        self.persons.get(pid as usize).is_some_and(|p| p.acquired)
    }

    pub(crate) fn person_duplicate(&self, pid: u32) -> bool {
        self.persons.get(pid as usize).is_some_and(|p| p.duplicate)
    }

    pub(crate) fn person_center_x(&self, pid: u32) -> f32 {
        self.persons
            .get(pid as usize)
            .map(|p| p.center_x())
            .unwrap_or(0.0)
    }

    pub(crate) fn person_family_node(&self, pid: u32) -> Option<NodeId> {
        self.persons.get(pid as usize).map(|p| {
            if let Some(f) = p.family {
                NodeId::Family(f)
            } else {
                NodeId::Person(pid)
            }
        })
    }
}
