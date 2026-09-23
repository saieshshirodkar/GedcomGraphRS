use crate::config::UNION_DISTANCE;
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn union_initialize_descendants(&mut self, uid: u32) {
        let generation = self
            .unions
            .get(uid as usize)
            .map(|u| u.generation)
            .unwrap_or(0);
        if generation >= 0 {
            return;
        }
        let mut chain: Vec<u32> = Vec::new();
        let mut cur = uid;
        loop {
            let g = self
                .unions
                .get(cur as usize)
                .map(|u| u.generation)
                .unwrap_or(0);
            if g >= -1 {
                break;
            }
            let nxt = self
                .unions
                .get(cur as usize)
                .and_then(|u| u.ancestor)
                .and_then(|a| self.node_youth(a))
                .and_then(|y| self.groups.get(y as usize))
                .and_then(|gr| gr.list.first().copied())
                .and_then(|n| self.node_union(n));
            if let Some(v) = nxt {
                chain.push(v);
                cur = v;
            } else {
                break;
            }
        }
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.descendants = chain;
        }
    }

    pub(crate) fn union_initialize_youths(&mut self, uid: u32) {
        let members = self
            .unions
            .get(uid as usize)
            .map(|u| u.list.clone())
            .unwrap_or_default();
        let mut out: Vec<u32> = Vec::new();
        for n in members {
            if let Some(y) = self.node_youth(n) {
                let mini = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(true);
                if !mini {
                    out.push(y);
                }
            }
        }
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.youths = out;
        }
    }

    pub(crate) fn union_place_origins_ascending(&mut self, uid: u32) {
        let origins = self.union_origins(uid);
        if origins.len() > 1 {
            let a = origins[0];
            let b = origins[1];
            let (au, bu) = (self.node_union(a), self.node_union(b));
            if let (Some(x), Some(y)) = (au, bu) {
                self.union_update_x(x);
                self.union_update_x(y);
                let xw = self.union_refresh_width(x);
                let xx = self.unions.get(x as usize).map(|u| u.x).unwrap_or(0.0);
                let yx = self.unions.get(y as usize).map(|u| u.x).unwrap_or(0.0);
                let overlap = xx + xw + UNION_DISTANCE - yx;
                if overlap > 0.0 {
                    self.union_move_ascending(x, -overlap / 2.0);
                    self.union_move_ascending(y, overlap / 2.0);
                }
            }
        }
    }

    pub(crate) fn union_move_ascending(&mut self, uid: u32, shift: f32) {
        self.union_update_x(uid);
        let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
        self.union_set_x(uid, ux + shift);
        let ancestor = self.unions.get(uid as usize).and_then(|u| u.ancestor);
        if let Some(a) = ancestor {
            for o in self.family_or_single_origins(a) {
                if let Some(ou) = self.node_union(o) {
                    self.union_move_ascending(ou, shift);
                }
            }
        }
    }

    pub(crate) fn family_or_single_origins(&self, nid: NodeId) -> Vec<NodeId> {
        match nid {
            NodeId::Person(i) => self
                .persons
                .get(i as usize)
                .and_then(|p| p.origin)
                .map(|o| vec![o])
                .unwrap_or_default(),
            NodeId::Family(_) => self.family_origins(nid),
        }
    }

    pub(crate) fn union_move_descending(&mut self, uid: u32, shift: f32) {
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.x += shift;
        }
        let members = self
            .unions
            .get(uid as usize)
            .map(|u| u.list.clone())
            .unwrap_or_default();
        for n in members {
            if let Some(y) = self.node_youth(n) {
                self.group_move_descending(y, shift);
            }
        }
    }
}
