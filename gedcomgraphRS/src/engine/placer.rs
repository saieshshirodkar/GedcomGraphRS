use crate::config::PROGENY_DISTANCE;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn place_nodes(&mut self) {
        for r in 0..self.union_rows.len() {
            let y_axe = self.union_rows.get(r).map(|x| x.y_axe).unwrap_or(0.0);
            let unions = self
                .union_rows
                .get(r)
                .map(|x| x.unions.clone())
                .unwrap_or_default();
            if let Some(row) = self.union_rows.get_mut(r) {
                row.y_axe = y_axe;
            }
            for uid in unions {
                let h = self.union_refresh_height(uid);
                if let Some(u) = self.unions.get_mut(uid as usize) {
                    u.y = y_axe - h / 2.0;
                }
                let members = self
                    .unions
                    .get(uid as usize)
                    .map(|u| u.list.clone())
                    .unwrap_or_default();
                for n in members {
                    let nh = self.node_height(n);
                    self.set_node_y(n, y_axe - nh / 2.0);
                }
            }
        }
        let gids: Vec<u32> = self.group_seq.clone();
        for gid in gids {
            let (mini, empty) = self
                .groups
                .get(gid as usize)
                .map(|g| (g.mini, g.list.is_empty()))
                .unwrap_or((true, true));
            if empty {
                continue;
            }
            if !mini && self.group_is_origin_mini_or_empty(gid) {
                let row_y = self
                    .union_rows
                    .get(self.row_slot(gid))
                    .map(|r| r.y_axe)
                    .unwrap_or(0.0);
                self.group_refresh_height(gid);
                self.group_place_origin_y(gid, row_y);
            }
            let members = self
                .groups
                .get(gid as usize)
                .map(|g| g.list.clone())
                .unwrap_or_default();
            for n in members {
                self.place_acquired_origin_y(n);
                if let Some(y) = self.node_youth(n) {
                    let ymin = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(false);
                    if ymin {
                        let ny = self.node_y(n);
                        let nh = self.node_height(n);
                        let kids = self
                            .groups
                            .get(y as usize)
                            .map(|g| g.list.clone())
                            .unwrap_or_default();
                        for k in kids {
                            self.set_node_y(k, ny + nh + PROGENY_DISTANCE);
                        }
                        if let Some(gr) = self.groups.get_mut(y as usize) {
                            gr.y = ny + nh + PROGENY_DISTANCE;
                        }
                    }
                }
            }
        }
        let fulcrum = self.fulcrum_group;
        self.group_place_nodes(fulcrum, 0.0);
        if self.max_above >= 0 {
            for r in (0..=self.max_above as usize).rev() {
                if r < self.group_rows.len() {
                    self.group_row_place_ancestors(r);
                }
            }
        }
        let start = (self.max_above - 1).max(0) as usize;
        for r in start..self.union_rows.len() {
            self.union_row_place_youths(r);
        }
        if self.max_above > 0 {
            for r in (0..self.max_above as usize).rev() {
                if r < self.union_rows.len() {
                    self.union_row_place_origins_ascending(r);
                }
            }
        }
        let mut count = 100i32;
        let mut forces = f32::MAX;
        while count > 0 && forces.abs() > 1.0 {
            self.clear_forces();
            if self.max_above > 0 {
                for r in (0..self.max_above as usize).rev() {
                    if r < self.union_rows.len() {
                        self.union_row_outdistance_ancestors(r);
                    }
                }
            }
            if self.max_above > 1 {
                for r in (0..self.max_above as usize - 1).rev() {
                    if r < self.union_rows.len() {
                        let unions = self
                            .union_rows
                            .get(r)
                            .map(|x| x.unions.clone())
                            .unwrap_or_default();
                        for uid in unions {
                            let shift = self.union_align_between_origins(uid);
                            let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
                            self.union_set_x(uid, ux + shift);
                        }
                    }
                }
            }
            let lo = (self.max_above - 1).max(0) as usize;
            for r in lo..self.union_rows.len() {
                self.union_row_outdistance_descendants(r);
            }
            forces = self.total_force();
            count -= 1;
        }
        for r in 0..self.union_rows.len() {
            self.union_row_resolve_overlap(r);
        }
        if self.max_above > 0 {
            let slot = self.max_above as usize - 1;
            let has_parent = self
                .union_rows
                .get(slot)
                .is_some_and(|r| !r.unions.is_empty());
            if has_parent {
                let puid = self
                    .union_rows
                    .get(slot)
                    .and_then(|r| r.unions.first().copied());
                if let Some(p) = puid {
                    let shift = self.union_align_between_origins(p);
                    self.union_move_descending(p, shift);
                }
            }
        }
        let all = self.nodes.clone();
        for nid in all {
            self.align_mini_empty_over_youth(nid);
            self.place_acquired_origin_x(nid);
            self.place_mini_children_x(nid);
        }
        self.normalize_origin();
        if !self.left_to_right {
            self.mirror_horizontal();
        }
        self.update_duplicate_lines();
        self.distribute_into(false);
        self.distribute_into(true);
    }

    fn row_slot(&self, gid: u32) -> usize {
        let generation = self
            .groups
            .get(gid as usize)
            .map(|g| g.generation)
            .unwrap_or(0);
        (generation + self.max_above).max(0) as usize
    }

    fn clear_forces(&mut self) {
        for p in &mut self.persons {
            p.base.force = 0.0;
        }
        for f in &mut self.families {
            f.base.force = 0.0;
        }
    }

    fn total_force(&self) -> f32 {
        let mut out = 0.0f32;
        for p in &self.persons {
            out += p.base.force;
        }
        for f in &self.families {
            out += f.base.force;
        }
        out
    }
}
