use crate::config::PROGENY_PLAY;
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn set_node_x(&mut self, nid: NodeId, x: f32) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.apply_x(x);
                }
            }
            NodeId::Family(i) => self.set_family_x(i, x),
        }
    }

    fn set_family_x(&mut self, fid: u32, x: f32) {
        let (side, partners, bond_w, overlap) = self
            .families
            .get(fid as usize)
            .map(|f| {
                (
                    f.side,
                    f.partners.clone(),
                    f.bond.as_ref().map(|b| b.w).unwrap_or(0.0),
                    f.bond.as_ref().map(|b| b.overlap).unwrap_or(0.0),
                )
            })
            .unwrap_or_default();
        if let Some(f) = self.families.get_mut(fid as usize) {
            f.base.force += x - f.base.x;
            f.base.x = x;
        }
        if partners.is_empty() {
            if let Some(f) = self.families.get_mut(fid as usize) {
                if let Some(b) = f.bond.as_mut() {
                    b.place_x(x);
                }
            }
            return;
        }
        if side == crate::config::Side::Right {
            if let Some(f) = self.families.get_mut(fid as usize) {
                if let Some(b) = f.bond.as_mut() {
                    b.x = x;
                }
            }
            if let Some(p0) = partners.first() {
                if let Some(p) = self.persons.get_mut(*p0 as usize) {
                    p.apply_x(x + bond_w - overlap);
                }
            }
            return;
        }
        let mut cursor = x;
        for (k, p0) in partners.iter().enumerate() {
            if let Some(p) = self.persons.get_mut(*p0 as usize) {
                p.apply_x(cursor);
            }
            if k == 0 {
                let w0 = self
                    .persons
                    .get(*p0 as usize)
                    .map(|p| p.base.w)
                    .unwrap_or(0.0);
                cursor += w0;
                if let Some(f) = self.families.get_mut(fid as usize) {
                    if let Some(b) = f.bond.as_mut() {
                        b.place_x(cursor);
                    }
                }
                cursor += self
                    .families
                    .get(fid as usize)
                    .map(|f| f.bond_width())
                    .unwrap_or(0.0);
            }
        }
    }

    pub(crate) fn set_node_y(&mut self, nid: NodeId, y: f32) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.apply_y(y);
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.y = y;
                }
                let cy = self.node_center_y(NodeId::Family(i));
                let partners = self
                    .families
                    .get(i as usize)
                    .map(|f| f.partners.clone())
                    .unwrap_or_default();
                for p0 in partners {
                    if let Some(p) = self.persons.get_mut(p0 as usize) {
                        let rel = p.center_rel_y();
                        p.apply_y(cy - rel);
                    }
                }
                if let Some(f) = self.families.get_mut(i as usize) {
                    if let Some(b) = f.bond.as_mut() {
                        b.y = y;
                    }
                }
            }
        }
    }

    pub(crate) fn move_descending(&mut self, nid: NodeId, shift: f32) {
        let youth = self.node_youth(nid);
        let nx = self.node_x(nid);
        self.set_node_x(nid, nx + shift);
        if let Some(y) = youth {
            let kids = self
                .groups
                .get(y as usize)
                .map(|g| g.list.clone())
                .unwrap_or_default();
            for k in kids {
                self.move_descending(k, shift);
            }
        }
    }

    pub(crate) fn place_youth_x(&mut self, nid: NodeId) {
        if let Some(y) = self.node_youth(nid) {
            let mini = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(true);
            if !mini {
                let cx = self.node_center_x(nid);
                self.group_place_nodes(y, cx);
            }
        }
    }

    pub(crate) fn set_node_raw_x(&mut self, nid: NodeId, x: f32) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.x = x;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.x = x;
                }
            }
        }
    }

    pub(crate) fn place_mini_children_x(&mut self, nid: NodeId) {
        if let Some(y) = self.node_youth(nid) {
            let mini = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(false);
            if mini {
                let cx = self.node_center_x(nid);
                let mut pos = cx;
                let kids = self
                    .groups
                    .get(y as usize)
                    .map(|g| g.list.clone())
                    .unwrap_or_default();
                for k in &kids {
                    self.set_node_raw_x(*k, pos);
                    pos += self.node_width(*k) + PROGENY_PLAY;
                }
                self.group_update_x(y);
                let gw = self.group_refresh_width(y);
                let diff = -gw / 2.0;
                for k in &kids {
                    let nx = self.node_x(*k);
                    self.set_node_x(*k, nx + diff);
                }
                if let Some(g) = self.groups.get_mut(y as usize) {
                    g.x += diff;
                }
            }
        }
    }
}
