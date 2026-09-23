use crate::config::{HORIZONTAL_SPACE, UNION_DISTANCE};
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;
use crate::model::gedcom::GedcomData;

impl Animator {
    pub(crate) fn gedcom_person_id(&self, ged: &GedcomData, pid: u32) -> String {
        self.persons
            .get(pid as usize)
            .and_then(|p| ged.person(p.person))
            .map(|g| g.id.clone())
            .unwrap_or_default()
    }

    pub(crate) fn node_label(&self, ged: &GedcomData, nid: NodeId) -> String {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.persons.get(i as usize) {
                    p.label(&crate::model::gedcom::essence(ged, Some(p.person)))
                } else {
                    String::new()
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.families.get(i as usize) {
                    let parts: Vec<String> = f
                        .partners
                        .iter()
                        .filter_map(|q| self.persons.get(*q as usize))
                        .map(|q| q.label(&crate::model::gedcom::essence(ged, Some(q.person))))
                        .collect();
                    f.describe(&parts)
                } else {
                    String::new()
                }
            }
        }
    }

    fn row_unions(&self, row: usize) -> Vec<u32> {
        self.union_rows
            .get(row)
            .map(|r| r.unions.clone())
            .unwrap_or_default()
    }

    fn row_members(&self, uid: u32) -> Vec<crate::core::node_base::NodeId> {
        self.unions
            .get(uid as usize)
            .map(|u| u.list.clone())
            .unwrap_or_default()
    }

    pub(crate) fn group_row_place_ancestors(&mut self, row: usize) {
        let groups = self
            .group_rows
            .get(row)
            .map(|r| r.groups.clone())
            .unwrap_or_default();
        for gid in groups {
            self.group_place_ancestors(gid);
        }
    }

    pub(crate) fn union_row_find_central(&mut self, row: usize) {
        let generation = self.union_rows.get(row).map(|r| r.generation).unwrap_or(0);
        let unions = self.row_unions(row);
        if unions.is_empty() {
            return;
        }
        if generation == -1 {
            let central = self.unions.get(unions[0] as usize).and_then(|u| u.ancestor);
            if let Some(r) = self.union_rows.get_mut(row) {
                r.central = central;
            }
            return;
        }
        let mid = self
            .unions
            .get(unions[unions.len() / 2] as usize)
            .map(|u| u.list.clone())
            .unwrap_or_default();
        let central = mid.get(mid.len() / 2).copied();
        if let Some(r) = self.union_rows.get_mut(row) {
            r.central = central;
        }
    }

    pub(crate) fn union_row_resolve_overlap(&mut self, row: usize) {
        let central = self.union_rows.get(row).and_then(|r| r.central);
        let Some(central_id) = central else { return };
        let mut left = central_id;
        while let Some(p) = self.node_prev(left) {
            let same = self.node_union(left) == self.node_union(p);
            let gap = if same {
                HORIZONTAL_SPACE
            } else {
                UNION_DISTANCE
            };
            let overlap = self.node_x(p) + self.node_width(p) + gap - self.node_x(left);
            if overlap > 0.0 {
                self.slide_node(p, -overlap);
            }
            left = p;
        }
        let mut right = central_id;
        while let Some(n) = self.node_next(right) {
            let same = self.node_union(right) == self.node_union(n);
            let gap = if same {
                HORIZONTAL_SPACE
            } else {
                UNION_DISTANCE
            };
            let overlap = self.node_x(right) + self.node_width(right) + gap - self.node_x(n);
            if overlap > 0.0 {
                self.slide_node(n, overlap);
            }
            right = n;
        }
    }

    pub(crate) fn union_row_place_origins_ascending(&mut self, row: usize) {
        for uid in self.row_unions(row) {
            self.union_place_origins_ascending(uid);
        }
    }

    pub(crate) fn union_row_outdistance_ancestors(&mut self, row: usize) {
        for uid in self.row_unions(row) {
            self.union_outdistance_ancestor_column(uid);
        }
    }

    pub(crate) fn union_row_outdistance_descendants(&mut self, row: usize) {
        for uid in self.row_unions(row) {
            for n in self.row_members(uid) {
                self.outdistance_descendant_column(n);
            }
            self.union_distribute_over_youth(uid);
        }
    }

    pub(crate) fn union_row_place_youths(&mut self, row: usize) {
        for uid in self.row_unions(row) {
            for n in self.row_members(uid) {
                self.place_youth_x(n);
            }
        }
    }
}
