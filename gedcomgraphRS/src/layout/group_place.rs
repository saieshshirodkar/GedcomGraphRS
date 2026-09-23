use crate::config::{ANCESTRY_DISTANCE, HORIZONTAL_SPACE};
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    fn group_list(&self, gid: u32) -> Vec<NodeId> {
        self.groups
            .get(gid as usize)
            .map(|g| g.list.clone())
            .unwrap_or_default()
    }

    pub(crate) fn group_place_nodes(&mut self, gid: u32, center_x: f32) {
        let left = self.group_basic_left_width(gid);
        let central = self.group_basic_central_width(gid);
        let mut pos = center_x - left - central / 2.0;
        for nid in self.group_list(gid) {
            self.set_node_x(nid, pos);
            pos += self.node_width(nid) + HORIZONTAL_SPACE;
        }
    }

    pub(crate) fn group_place_ancestors(&mut self, gid: u32) {
        self.group_place_origin_x(gid);
        let origin = self.groups.get(gid as usize).and_then(|g| g.origin);
        if let Some(o) = origin {
            if let Some(uid) = self.node_union(o) {
                let members = self
                    .unions
                    .get(uid as usize)
                    .map(|u| u.list.clone())
                    .unwrap_or_default();
                let pos = members.iter().position(|n| *n == o).unwrap_or(0);
                let ox = self.node_x(o);
                let mut cursor = ox;
                for n in members[..pos].iter().rev() {
                    cursor -= self.node_width(*n) + HORIZONTAL_SPACE;
                    self.set_node_x(*n, cursor);
                }
                cursor = ox + self.node_width(o) + HORIZONTAL_SPACE;
                for n in &members[pos + 1..] {
                    self.set_node_x(*n, cursor);
                    cursor += self.node_width(*n) + HORIZONTAL_SPACE;
                }
            }
        }
    }

    pub(crate) fn group_place_origin_x(&mut self, gid: u32) {
        let (first, last, origin) = self
            .groups
            .get(gid as usize)
            .map(|g| (g.first, g.last, g.origin))
            .unwrap_or((Option::None, Option::None, Option::None));
        if let (Some(f), Some(l), Some(o)) = (first, last, origin) {
            let fx = self.person_center_x(f);
            let lx = self.person_center_x(l);
            let rel = self.node_center_rel_x(o);
            self.set_node_x(o, fx + (lx - fx) / 2.0 - rel);
        }
    }

    pub(crate) fn group_place_origin_y(&mut self, gid: u32, row_y: f32) {
        let (first, last, origin) = self
            .groups
            .get(gid as usize)
            .map(|g| (g.first, g.last, g.origin))
            .unwrap_or((Option::None, Option::None, Option::None));
        if let (Some(f), Some(l), Some(o)) = (first, last, origin) {
            let gap = if f == l {
                ANCESTRY_DISTANCE
            } else {
                self.spacing.little_calc
            };
            if let Some(g) = self.groups.get_mut(gid as usize) {
                g.y = row_y - g.h / 2.0;
            }
            let gy = self.groups.get(gid as usize).map(|g| g.y).unwrap_or(0.0);
            let oh = self.node_height(o);
            self.set_node_y(o, gy - gap - oh);
        }
    }

    pub(crate) fn group_move_descending(&mut self, gid: u32, shift: f32) {
        let mini = self
            .groups
            .get(gid as usize)
            .map(|g| g.mini)
            .unwrap_or(true);
        if mini {
            return;
        }
        self.group_update_x(gid);
        let gx = self.groups.get(gid as usize).map(|g| g.x).unwrap_or(0.0);
        if let Some(g) = self.groups.get_mut(gid as usize) {
            g.x = gx + shift;
        }
        for nid in self.group_list(gid) {
            if let Some(y) = self.node_youth(nid) {
                self.group_move_descending(y, shift);
            }
        }
    }

    pub(crate) fn group_update_x(&mut self, gid: u32) {
        let first_x = self
            .groups
            .get(gid as usize)
            .and_then(|g| g.list.first().copied())
            .map(|n| self.node_x(n))
            .unwrap_or(0.0);
        if let Some(g) = self.groups.get_mut(gid as usize) {
            g.x = first_x;
        }
    }

    pub(crate) fn group_refresh_width(&mut self, gid: u32) -> f32 {
        let (first_x, last_right) = self
            .groups
            .get(gid as usize)
            .and_then(|g| g.list.first().copied().zip(g.list.last().copied()))
            .map(|(a, b)| (self.node_x(a), self.node_x(b) + self.node_width(b)))
            .unwrap_or((0.0, 0.0));
        let w = last_right - first_x;
        if let Some(g) = self.groups.get_mut(gid as usize) {
            g.x = first_x;
            g.w = w;
        }
        w
    }

    pub(crate) fn group_refresh_height(&mut self, gid: u32) -> f32 {
        let mut h = 0.0f32;
        for nid in self.group_list(gid) {
            h = h.max(self.node_height(nid));
        }
        if let Some(g) = self.groups.get_mut(gid as usize) {
            g.h = h;
        }
        h
    }

    pub(crate) fn group_center_rel_x(&self, gid: u32) -> f32 {
        if let Some(g) = self.groups.get(gid as usize) {
            if let (Some(f), Some(l)) = (g.first, g.last) {
                let fx = self
                    .persons
                    .get(f as usize)
                    .map(|p| p.base.x + p.base.w / 2.0)
                    .unwrap_or(0.0);
                let lx = self
                    .persons
                    .get(l as usize)
                    .map(|p| p.base.x + p.base.w / 2.0)
                    .unwrap_or(0.0);
                return fx - g.x + (lx - fx) / 2.0;
            }
        }
        0.0
    }

    pub(crate) fn group_center_x(&self, gid: u32) -> f32 {
        self.groups.get(gid as usize).map(|g| g.x).unwrap_or(0.0) + self.group_center_rel_x(gid)
    }
}
