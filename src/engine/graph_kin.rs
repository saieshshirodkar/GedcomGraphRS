use crate::config::{Card, Match, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;
use crate::model::gedcom::PersonId;
use crate::model::kin::spouses_of;
use crate::nodes::family::FamilyNodeData;

impl Graph {
    pub(crate) fn find_acquired_ancestry(&mut self, pid: u32) {
        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
            p.acquired = true;
        }
    }
    pub(crate) fn create_node_from_family(
        &mut self,
        spouse_family: u32,
        generation: i32,
        kind: Card,
    ) -> u32 {
        let mini = kind == Card::Ancestry;
        let fid = self.anim.alloc_family(FamilyNodeData::of(
            spouse_family,
            mini,
            Side::None,
            self.left_to_right,
            generation,
        ));
        if kind == Card::Regular || self.with_numbers {
            if let Some(f) = self.anim.families.get_mut(fid as usize) {
                f.base.stamp = Match::Main;
            }
            let spouses = spouses_of(&self.gedcom, spouse_family, Option::None);
            for s in spouses {
                let q = self.make_person_node(s.0, kind, generation);
                self.check_for_duplicate(q, Some(spouse_family));
                self.family_add_partner(fid, q);
            }
        }
        self.family_create_bond(fid);
        self.anim.add_node(NodeId::Family(fid));
        fid
    }

    pub(crate) fn create_next_family_node(
        &mut self,
        spouse_family: u32,
        excluded: u32,
        generation: i32,
        side: Side,
        stamp: Match,
        parent: Option<NodeId>,
    ) -> u32 {
        let fid = self.anim.alloc_family(FamilyNodeData::of(
            spouse_family,
            false,
            side,
            self.left_to_right,
            generation,
        ));
        if let Some(f) = self.anim.families.get_mut(fid as usize) {
            f.base.stamp = stamp;
        }
        if self.with_spouses {
            let partners = spouses_of(&self.gedcom, spouse_family, Some(PersonId(excluded)));
            for s in partners {
                let q = self.make_person_node(s.0, Card::Regular, generation);
                let same_parent = parent.and_then(|par| self.anim_parents_share(par, s.0));
                if same_parent.unwrap_or(false) {
                    if let Some(p) = self.anim.persons.get_mut(q as usize) {
                        p.origin = parent;
                    }
                } else {
                    self.find_acquired_ancestry(q);
                }
                self.family_add_partner(fid, q);
                self.check_for_duplicate(q, Some(spouse_family));
            }
        }
        self.family_create_bond(fid);
        self.anim.add_node(NodeId::Family(fid));
        fid
    }
}
