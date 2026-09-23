use crate::config::HORIZONTAL_SPACE;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn union_align_between_origins(&mut self, uid: u32) -> f32 {
        let origins = self.union_origins(uid);
        if origins.len() > 1 {
            let (a, b) = (origins[0], origins[1]);
            let ax = self.node_center_x(a);
            let bx = self.node_center_x(b);
            self.union_update_x(uid);
            let ay = self.node_youth(a);
            let by = self.node_youth(b);
            if let (Some(x), Some(y)) = (ay, by) {
                self.group_refresh_width(x);
                self.group_refresh_width(y);
                let xc = self.group_center_x(x);
                let yc = self.group_center_x(y);
                let youths_distance = yc - xc;
                let discrepancy = bx - ax - youths_distance;
                let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
                let youth_rel = self.group_center_rel_x(x);
                return ax - youth_rel - ux + discrepancy / 2.0;
            }
            return 0.0;
        }
        if let Some(o) = origins.first() {
            if self.node_union(*o).is_some() {
                if let Some(y) = self.node_youth(*o) {
                    self.group_refresh_width(y);
                    let yc = self.group_center_x(y);
                    return self.node_center_x(*o) - yc;
                }
            }
        }
        0.0
    }

    pub(crate) fn union_distribute_over_youth(&mut self, uid: u32) {
        let youths = self
            .unions
            .get(uid as usize)
            .map(|u| u.youths.clone())
            .unwrap_or_default();
        if youths.is_empty() {
            return;
        }
        for y in &youths {
            self.group_refresh_width(*y);
            let yc = self.group_center_x(*y);
            if let Some(o) = self.groups.get(*y as usize).and_then(|g| g.origin) {
                let rel = self.node_center_rel_x(o);
                self.set_node_x(o, yc - rel);
            }
        }
        if let Some(first_origin) = self.groups.get(youths[0] as usize).and_then(|g| g.origin) {
            let mut node = first_origin;
            while let Some(p) = self.node_prev(node) {
                if self.node_union(p) != Some(uid) {
                    break;
                }
                let nx = self.node_x(node) - HORIZONTAL_SPACE - self.node_width(p);
                self.set_node_x(p, nx);
                node = p;
            }
        }
        for w in youths.windows(2) {
            let (left_y, right_y) = (w[0], w[1]);
            let (lo, ro) = (
                self.groups.get(left_y as usize).and_then(|g| g.origin),
                self.groups.get(right_y as usize).and_then(|g| g.origin),
            );
            if let (Some(l), Some(r)) = (lo, ro) {
                let mut node = self.node_next(l);
                while let Some(n) = node {
                    if n == r {
                        break;
                    }
                    let nxt = self.node_next(n);
                    if let (Some(p), Some(q)) = (self.node_prev(n), nxt) {
                        let space = self.node_x(q)
                            - self.node_x(p)
                            - self.node_width(p)
                            - self.node_width(n);
                        let px = self.node_x(p);
                        let pw = self.node_width(p);
                        self.set_node_x(n, px + pw + space / 2.0);
                    }
                    node = nxt;
                    if node == Some(r) {
                        break;
                    }
                }
            }
        }
        if let Some(last_origin) = self
            .groups
            .get(*youths.last().unwrap_or(&0) as usize)
            .and_then(|g| g.origin)
        {
            let mut node = last_origin;
            while let Some(n) = self.node_next(node) {
                if self.node_union(n) != Some(uid) {
                    break;
                }
                let nx = self.node_x(node) + self.node_width(node) + HORIZONTAL_SPACE;
                self.set_node_x(n, nx);
                node = n;
            }
        }
    }
}
