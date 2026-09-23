use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn base_of(&self, nid: NodeId) -> (f32, f32, f32, f32, i32, bool) {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .map(|p| {
                    (
                        p.base.x,
                        p.base.y,
                        p.base.w,
                        p.base.h,
                        p.base.generation,
                        p.base.mini,
                    )
                })
                .unwrap_or_default(),
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .map(|f| {
                    (
                        f.base.x,
                        f.base.y,
                        f.base.w,
                        f.base.h,
                        f.base.generation,
                        f.base.mini,
                    )
                })
                .unwrap_or_default(),
        }
    }

    pub(crate) fn node_x(&self, nid: NodeId) -> f32 {
        self.base_of(nid).0
    }

    pub(crate) fn node_y(&self, nid: NodeId) -> f32 {
        self.base_of(nid).1
    }

    pub(crate) fn node_width(&self, nid: NodeId) -> f32 {
        self.base_of(nid).2
    }

    pub(crate) fn node_height(&self, nid: NodeId) -> f32 {
        self.base_of(nid).3
    }

    pub(crate) fn node_generation(&self, nid: NodeId) -> i32 {
        self.base_of(nid).4
    }

    pub(crate) fn node_mini(&self, nid: NodeId) -> bool {
        self.base_of(nid).5
    }

    pub(crate) fn node_union(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.base.union),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| f.base.union),
        }
    }

    pub(crate) fn node_group(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.base.group),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| f.base.group),
        }
    }

    pub(crate) fn node_youth(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.base.youth),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| f.base.youth),
        }
    }

    pub(crate) fn node_prev(&self, nid: NodeId) -> Option<NodeId> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.base.prev),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| f.base.prev),
        }
    }

    pub(crate) fn node_next(&self, nid: NodeId) -> Option<NodeId> {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).and_then(|p| p.base.next),
            NodeId::Family(i) => self.families.get(i as usize).and_then(|f| f.base.next),
        }
    }

    pub(crate) fn node_is_ancestor(&self, nid: NodeId) -> bool {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .is_some_and(|p| p.base.is_ancestor),
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .is_some_and(|f| f.base.is_ancestor),
        }
    }

    pub(crate) fn node_married_siblings(&self, nid: NodeId) -> bool {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .is_some_and(|p| p.base.married_siblings),
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .is_some_and(|f| f.base.married_siblings),
        }
    }

    pub(crate) fn node_is_multi(&self, nid: NodeId) -> bool {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .is_some_and(|p| p.base.stamp.is_multi()),
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .is_some_and(|f| f.base.stamp.is_multi()),
        }
    }

    pub(crate) fn node_is_duplicate(&self, nid: NodeId) -> bool {
        match nid {
            NodeId::Person(i) => self.persons.get(i as usize).is_some_and(|p| p.duplicate),
            NodeId::Family(i) => self
                .families
                .get(i as usize)
                .is_some_and(|f| f.partners.iter().any(|p| self.person_duplicate(*p))),
        }
    }
}
