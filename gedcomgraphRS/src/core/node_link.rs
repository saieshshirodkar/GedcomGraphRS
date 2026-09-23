use crate::config::Branch;
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn set_node_group(&mut self, nid: NodeId, gid: Option<u32>) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.group = gid;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.group = gid;
                }
            }
        }
    }

    pub(crate) fn set_node_youth(&mut self, nid: NodeId, youth: Option<u32>) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.youth = youth;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.youth = youth;
                }
            }
        }
    }

    pub(crate) fn set_node_union(&mut self, nid: NodeId, union: Option<u32>) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.union = union;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.union = union;
                }
            }
        }
    }

    pub(crate) fn set_node_prev(&mut self, nid: NodeId, prev: Option<NodeId>) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.prev = prev;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.prev = prev;
                }
            }
        }
    }

    pub(crate) fn set_node_next(&mut self, nid: NodeId, next: Option<NodeId>) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.next = next;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.next = next;
                }
            }
        }
    }

    pub(crate) fn node_left_width(&self, nid: NodeId, branch: Branch) -> f32 {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .map(|p| p.center_rel_x())
                .unwrap_or(0.0),
            NodeId::Family(i) => self.family_left_width(i, branch),
        }
    }

    fn family_left_width(&self, fid: u32, branch: Branch) -> f32 {
        if let Some(f) = self.families.get(fid as usize) {
            let main_pos = self
                .node_main_person(NodeId::Family(fid))
                .and_then(|m| f.partners.iter().position(|p| *p == m));
            if (branch == Branch::Mater || main_pos.is_some_and(|v| v > 0)) && f.partners.len() > 1
            {
                let a = f.partners.first().copied().unwrap_or(u32::MAX);
                let b = f.partners.get(1).copied().unwrap_or(u32::MAX);
                let aw = self
                    .persons
                    .get(a as usize)
                    .map(|p| p.base.w)
                    .unwrap_or(0.0);
                let bx = self
                    .persons
                    .get(b as usize)
                    .map(|p| p.center_rel_x())
                    .unwrap_or(0.0);
                return aw
                    + self
                        .families
                        .get(fid as usize)
                        .map(|x| x.bond_width())
                        .unwrap_or(0.0)
                    + bx;
            }
            if let Some(p0) = f.partners.first() {
                return self
                    .persons
                    .get(*p0 as usize)
                    .map(|p| p.center_rel_x())
                    .unwrap_or(0.0);
            }
        }
        0.0
    }

    pub(crate) fn node_center_rel_x(&self, nid: NodeId) -> f32 {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .map(|p| p.center_rel_x())
                .unwrap_or(0.0),
            NodeId::Family(i) => self.family_center_rel_x(i),
        }
    }

    fn family_center_rel_x(&self, fid: u32) -> f32 {
        if let Some(f) = self.families.get(fid as usize) {
            if f.partners.is_empty() || f.side == crate::config::Side::Right {
                return f.bond.as_ref().map(|b| b.w / 2.0).unwrap_or(0.0);
            }
            if f.partners.len() > 1 || f.side == crate::config::Side::Left {
                let w0 = f
                    .partners
                    .first()
                    .and_then(|p| self.persons.get(*p as usize))
                    .map(|p| p.base.w)
                    .unwrap_or(0.0);
                return w0 + f.bond_width() / 2.0;
            }
            if let Some(p0) = f
                .partners
                .first()
                .and_then(|p| self.persons.get(*p as usize))
            {
                return p0.base.w / 2.0;
            }
        }
        0.0
    }

    pub(crate) fn node_center_x(&self, nid: NodeId) -> f32 {
        self.node_x(nid) + self.node_center_rel_x(nid)
    }

    pub(crate) fn node_center_rel_y(&self, nid: NodeId) -> f32 {
        self.node_height(nid) / 2.0
    }

    pub(crate) fn node_center_y(&self, nid: NodeId) -> f32 {
        self.node_y(nid) + self.node_center_rel_y(nid)
    }
}
