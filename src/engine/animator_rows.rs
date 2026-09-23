use crate::config::Match;
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;
use crate::lines::segments::{LineKind, LineSeg};

impl Animator {
    pub(crate) fn build_lines(&mut self) {
        self.lines.clear();
        self.back_lines.clear();
        let order = self.nodes.clone();
        for nid in order {
            for p in self.node_persons(nid) {
                if let Some(o) = self.persons.get(p as usize).and_then(|q| q.origin) {
                    self.lines.push(LineSeg::of(LineKind::Curve {
                        person: p,
                        origin: o,
                    }));
                }
            }
            if let NodeId::Family(i) = nid {
                let (partners, stamp, has_bond, side, ltr) = self
                    .families
                    .get(i as usize)
                    .map(|f| {
                        (
                            f.partners.clone(),
                            f.base.stamp,
                            f.bond.is_some(),
                            f.side,
                            f.left_to_right,
                        )
                    })
                    .unwrap_or_default();
                if !partners.is_empty() && stamp != Match::Main {
                    if let Some(p0) = partners.first() {
                        self.lines.push(LineSeg::of(LineKind::Next {
                            owner: i,
                            partner: *p0,
                            side,
                            ltr,
                        }));
                    }
                } else if partners.len() > 1 {
                    if let (Some(a), Some(b)) = (partners.first(), partners.get(1)) {
                        self.lines.push(LineSeg::of(LineKind::Horizontal {
                            left: *a,
                            right: *b,
                            ltr,
                        }));
                    }
                }
                if stamp == Match::Near {
                    self.lines.push(LineSeg::of(LineKind::Back {
                        owner: i,
                        node: i,
                        side,
                        stamp,
                        no_partners: partners.is_empty(),
                        ltr,
                    }));
                } else if stamp == Match::Middle || stamp == Match::Far {
                    self.back_lines.push(LineSeg::of(LineKind::Back {
                        owner: i,
                        node: i,
                        side,
                        stamp,
                        no_partners: partners.is_empty(),
                        ltr,
                    }));
                }
                let kids = self.node_youth(nid).is_some()
                    || self
                        .families
                        .get(i as usize)
                        .map(|f| f.base.mini)
                        .unwrap_or(false);
                if kids && has_bond {
                    self.lines
                        .push(LineSeg::of(LineKind::Vertical { owner: i }));
                }
            }
        }
    }

    pub(crate) fn build_group_rows(&mut self, max_above: i32) {
        for gid in self.group_seq.clone() {
            let (mini, empty, generation, single_empty_ancestor) = self
                .groups
                .get(gid as usize)
                .map(|g| {
                    let single = g.list.len() == 1
                        && g.generation < 0
                        && g.list
                            .first()
                            .is_some_and(|n| self.node_person_count(*n) == 0);
                    (g.mini, g.list.is_empty(), g.generation, single)
                })
                .unwrap_or((true, true, 0, true));
            if mini || empty || single_empty_ancestor {
                continue;
            }
            let slot = (generation + max_above) as usize;
            if let Some(row) = self.group_rows.get_mut(slot) {
                row.groups.push(gid);
            }
        }
    }

    pub(crate) fn link_row_nodes(&mut self) {
        for r in 0..self.group_rows.len() {
            let groups = self
                .group_rows
                .get(r)
                .map(|x| x.groups.clone())
                .unwrap_or_default();
            let mut previous: Option<NodeId> = Option::None;
            for gid in groups {
                let list = self
                    .groups
                    .get(gid as usize)
                    .map(|g| g.list.clone())
                    .unwrap_or_default();
                for nid in list {
                    if Some(nid) != previous {
                        self.set_node_prev(nid, previous);
                        if let Some(p) = previous {
                            self.set_node_next(p, Some(nid));
                        }
                        previous = Some(nid);
                    }
                }
            }
        }
    }
}
