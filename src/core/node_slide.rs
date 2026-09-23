use crate::config::{ANCESTRY_DISTANCE, HORIZONTAL_SPACE, UNION_DISTANCE};
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn slide_node(&mut self, nid: NodeId, shift: f32) {
        self.set_node_x(nid, self.node_x(nid) + shift);
        if shift > 0.0 {
            if let Some(next) = self.node_next(nid) {
                let same = self.node_union(nid) == self.node_union(next);
                let gap = if same {
                    HORIZONTAL_SPACE
                } else {
                    UNION_DISTANCE
                };
                let over = self.node_x(nid) + self.node_width(nid) + gap - self.node_x(next);
                if over > 0.0 {
                    self.slide_node(next, over);
                }
            }
        } else if shift < 0.0 {
            if let Some(prev) = self.node_prev(nid) {
                let same = self.node_union(nid) == self.node_union(prev);
                let gap = if same {
                    HORIZONTAL_SPACE
                } else {
                    UNION_DISTANCE
                };
                let over = self.node_x(prev) + self.node_width(prev) + gap - self.node_x(nid);
                if over > 0.0 {
                    self.slide_node(prev, -over);
                }
            }
        }
    }

    pub(crate) fn initialize_origins(&mut self, nid: NodeId) {
        let generation = self.node_generation(nid);
        let mini = self.node_mini(nid);
        if generation < -1 || mini {
            return;
        }
        let mut chain: Vec<NodeId> = Vec::new();
        let mut cur: Option<NodeId> = Some(nid);
        while let Some(c) = cur {
            if self.node_generation(c) < 0 {
                break;
            }
            let g = self.node_group(c);
            let nxt = g.and_then(|gid| self.groups.get(gid as usize).and_then(|gr| gr.origin));
            cur = nxt;
            if let Some(n) = cur {
                if !self.node_mini(n) {
                    chain.push(n);
                }
            }
        }
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get_mut(i as usize) {
                    p.base.origins = Some(chain);
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get_mut(i as usize) {
                    f.base.origins = Some(chain);
                }
            }
        }
    }

    pub(crate) fn outdistance_descendant_column(&mut self, nid: NodeId) {
        if let Some(y) = self.node_youth(nid) {
            let mini = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(true);
            if mini {
                return;
            }
            let mut shift = 0.0f32;
            if let Some(prev) = self.node_prev(nid) {
                if self.node_union(nid) == self.node_union(prev) {
                    shift = self.node_x(prev) + self.node_width(prev) + HORIZONTAL_SPACE
                        - self.node_x(nid);
                }
            }
            shift = self.find_descendant_shift(nid, nid, shift);
            if shift != 0.0 {
                self.move_descending(nid, shift);
            }
        }
    }

    fn find_descendant_shift(&self, outer: NodeId, node: NodeId, mut shift: f32) -> f32 {
        if let Some(y) = self.node_youth(node) {
            let mini = self.groups.get(y as usize).map(|g| g.mini).unwrap_or(true);
            if !mini {
                let kids = self
                    .groups
                    .get(y as usize)
                    .map(|g| g.list.clone())
                    .unwrap_or_default();
                if let Some(first) = kids.first() {
                    if let Some(fp) = self.node_prev(*first) {
                        let prev_origins: Option<Vec<NodeId>> = match fp {
                            NodeId::Person(i) => self
                                .persons
                                .get(i as usize)
                                .and_then(|p| p.base.origins.clone()),
                            NodeId::Family(i) => self
                                .families
                                .get(i as usize)
                                .and_then(|f| f.base.origins.clone()),
                        };
                        if prev_origins.is_some_and(|o| !o.contains(&outer)) {
                            let left = self.node_x(fp) + self.node_width(fp) + UNION_DISTANCE
                                - self.node_x(*first);
                            if left > shift {
                                shift = left;
                            }
                        }
                    }
                }
                for k in kids {
                    shift = self.find_descendant_shift(outer, k, shift);
                }
            }
        }
        shift
    }
    pub(crate) fn place_acquired_origin_x(&mut self, nid: NodeId) {
        if let NodeId::Family(i) = nid {
            let partners = self
                .families
                .get(i as usize)
                .map(|f| f.partners.clone())
                .unwrap_or_default();
            for p0 in partners {
                let (acq, origin) = self
                    .persons
                    .get(p0 as usize)
                    .map(|p| (p.acquired, p.origin))
                    .unwrap_or((false, Option::None));
                if acq {
                    if let Some(o) = origin {
                        let cx = self
                            .persons
                            .get(p0 as usize)
                            .map(|p| p.center_x())
                            .unwrap_or(0.0);
                        let rel = self.node_center_rel_x(o);
                        self.set_node_x(o, cx - rel);
                    }
                }
            }
        }
    }

    pub(crate) fn place_acquired_origin_y(&mut self, nid: NodeId) {
        if let NodeId::Family(i) = nid {
            let mini = self
                .families
                .get(i as usize)
                .map(|f| f.base.mini)
                .unwrap_or(true);
            if mini {
                return;
            }
            let partners = self
                .families
                .get(i as usize)
                .map(|f| f.partners.clone())
                .unwrap_or_default();
            let ny = self.node_y(nid);
            for p0 in partners {
                let (acq, origin) = self
                    .persons
                    .get(p0 as usize)
                    .map(|p| (p.acquired, p.origin))
                    .unwrap_or((false, Option::None));
                if acq {
                    if let Some(o) = origin {
                        let oh = self.node_height(o);
                        self.set_node_y(o, ny - ANCESTRY_DISTANCE - oh);
                    }
                }
            }
        }
    }

    pub(crate) fn align_mini_empty_over_youth(&mut self, nid: NodeId) {
        if let Some(y) = self.node_youth(nid) {
            if self.group_is_origin_mini_or_empty(y) {
                self.group_refresh_width(y);
                let cx = self.group_center_x(y);
                let rel = self.node_center_rel_x(nid);
                self.set_node_x(nid, cx - rel);
            }
        }
    }
}
