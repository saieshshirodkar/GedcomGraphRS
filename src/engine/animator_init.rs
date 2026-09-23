use crate::config::{BOND_WIDTH, MARRIAGE_WIDTH, MINI_BOND_WIDTH, MINI_CARD_HEIGHT};
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;
use crate::layout::rows::{GroupRow, UnionRow};

impl Animator {
    pub(crate) fn init_nodes(
        &mut self,
        fulcrum_group: u32,
        max_above: i32,
        max_below: i32,
        with_numbers: bool,
    ) {
        self.fulcrum_group = fulcrum_group;
        self.max_above = max_above;
        self.width = 0.0;
        self.height = 0.0;
        self.biggest_path_size = 0.0;
        let total = (max_above + 1 + max_below).max(1) as usize;
        let mut row_max: Vec<f32> = vec![0.0; total];
        let order = self.nodes.clone();
        for nid in &order {
            if let NodeId::Family(i) = nid {
                self.size_family(*i);
            }
            if let Some(h) = self.node_row_height(*nid) {
                let slot = self.node_generation(*nid) + max_above;
                if slot >= 0 {
                    if let Some(r) = row_max.get_mut(slot as usize) {
                        if h > *r {
                            *r = h;
                        }
                    }
                }
            }
        }
        self.build_rows(max_above, max_below, &row_max);
        let gids: Vec<u32> = self.group_seq.clone();
        for gid in gids {
            self.group_set_origin(gid);
        }
        if !with_numbers {
            self.prune_mini_origins();
        }
        self.build_lines();
        self.build_group_rows(max_above);
        self.link_row_nodes();
        self.build_unions();
        let urows: Vec<usize> = (0..self.union_rows.len()).collect();
        for r in urows {
            self.union_row_find_central(r);
            self.link_union_row(r);
            let unions = self
                .union_rows
                .get(r)
                .map(|x| x.unions.clone())
                .unwrap_or_default();
            for uid in unions {
                self.union_initialize_descendants(uid);
                self.union_initialize_youths(uid);
            }
        }
        let all = self.nodes.clone();
        for nid in all {
            self.initialize_origins(nid);
        }
    }

    fn node_row_height(&self, nid: NodeId) -> Option<f32> {
        if self.node_mini(nid) || self.node_person_count(nid) == 0 {
            return Option::None;
        }
        Some(self.node_height(nid))
    }

    fn size_family(&mut self, fid: u32) {
        let partners = self
            .families
            .get(fid as usize)
            .map(|f| f.partners.clone())
            .unwrap_or_default();
        let mut w = 0.0f32;
        let mut h = 0.0f32;
        for p in &partners {
            if let Some(q) = self.persons.get(*p as usize) {
                w += q.base.w;
                h = h.max(q.base.h);
            }
        }
        if let Some(f) = self.families.get_mut(fid as usize) {
            f.base.w = w;
            f.base.h = h;
            if f.base.h == 0.0 {
                f.base.h = MINI_CARD_HEIGHT;
            }
        }
        if self
            .families
            .get(fid as usize)
            .and_then(|f| f.bond.as_ref())
            .is_some()
        {
            let mini = self
                .families
                .get(fid as usize)
                .map(|f| f.base.mini)
                .unwrap_or(false);
            let side = self
                .families
                .get(fid as usize)
                .map(|f| f.side)
                .unwrap_or(crate::config::Side::None);
            let dated = self
                .families
                .get(fid as usize)
                .and_then(|f| f.bond.as_ref())
                .is_some_and(|b| b.marriage_date.is_some());
            let bw = if dated {
                MARRIAGE_WIDTH
            } else if mini {
                MINI_BOND_WIDTH
            } else {
                BOND_WIDTH
            };
            let bh = self
                .families
                .get(fid as usize)
                .map(|f| f.base.h)
                .unwrap_or(0.0);
            if let Some(f) = self.families.get_mut(fid as usize) {
                if let Some(b) = f.bond.as_mut() {
                    b.w = bw;
                    b.h = bh;
                    if b.marriage_date.is_some() {
                        b.overlap = (MARRIAGE_WIDTH - crate::config::MARRIAGE_INNER_WIDTH) / 2.0;
                        if side == crate::config::Side::Left || side == crate::config::Side::Right {
                            f.base.w += b.overlap;
                        }
                    }
                    f.base.w += f.bond_width();
                }
            }
            self.bond_owners.push(fid);
        }
    }

    fn build_rows(&mut self, max_above: i32, max_below: i32, row_max: &[f32]) {
        self.union_rows.clear();
        self.group_rows.clear();
        let total = max_above + 1 + max_below;
        let mut pos = row_max.first().copied().unwrap_or(0.0) / 2.0;
        for generation in -max_above..total - max_above {
            self.union_rows.push(UnionRow::of(generation, pos));
            self.group_rows.push(GroupRow::of(generation));
            let a = (generation + max_above) as usize;
            if generation + max_above < total - 1 {
                let upper = row_max.get(a).copied().unwrap_or(0.0) / 2.0;
                let lower = row_max.get(a + 1).copied().unwrap_or(0.0) / 2.0;
                pos += upper + self.spacing.vertical_calc + lower;
            }
        }
    }

    fn prune_mini_origins(&mut self) {
        let mut drop_nodes: Vec<NodeId> = Vec::new();
        let mut drop_bonds: Vec<u32> = Vec::new();
        for i in 0..self.persons.len() {
            let origin = self.persons.get(i).and_then(|p| p.origin);
            if let Some(o) = origin {
                if self.node_mini(o) {
                    if let Some(y) = self.node_youth(o) {
                        let single = self
                            .groups
                            .get(y as usize)
                            .map(|g| g.list.len() == 1)
                            .unwrap_or(false);
                        if single {
                            if let Some(p) = self.persons.get_mut(i) {
                                p.origin = Option::None;
                            }
                            drop_nodes.push(o);
                            if let NodeId::Family(f) = o {
                                drop_bonds.push(f);
                            }
                        }
                    }
                }
            }
        }
        self.nodes.retain(|n| !drop_nodes.contains(n));
        self.bond_owners.retain(|b| !drop_bonds.contains(b));
    }
}
