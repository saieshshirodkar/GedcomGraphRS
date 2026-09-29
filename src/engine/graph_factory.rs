use crate::config::{Card, Match, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;
use crate::model::gedcom::FamilyId;
use crate::model::helpers::person_is_dead;
use crate::model::kin::{count_ancestors, count_descendants, family_marriage_date, spouses_of};
use crate::nodes::family::{BondData, FamilyNodeData};
use crate::nodes::person::PersonNodeData;

impl Graph {
    pub(crate) fn make_person_node(&mut self, person: u32, kind: Card, generation: i32) -> u32 {
        let mut node = PersonNodeData::single(person, kind, generation);
        if kind == Card::Fulcrum || kind == Card::Regular {
            if person_is_dead(&self.gedcom, person) {
                node.dead = true;
            }
        } else if kind == Card::Ancestry {
            node.amount = count_ancestors(&self.gedcom, person);
        } else if kind == Card::Progeny {
            node.amount = count_descendants(&self.gedcom, person);
        }
        self.anim.alloc_person(node)
    }

    pub(crate) fn create_node_from_person(
        &mut self,
        person: u32,
        spouse_family: Option<u32>,
        parent: Option<NodeId>,
        generation: i32,
        kind: Card,
        stamp: Match,
    ) -> NodeId {
        let pid = self.make_person_node(person, kind, generation);
        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
            p.base.stamp = stamp;
            p.origin = parent;
        }
        self.check_for_duplicate(pid, spouse_family);
        let mut family_id: Option<u32> = Option::None;
        let build_couple = (kind == Card::Fulcrum || kind == Card::Regular)
            && spouse_family.is_some()
            && !self
                .anim
                .persons
                .get(pid as usize)
                .is_some_and(|p| p.duplicate);
        if build_couple {
            let fam = spouse_family.unwrap_or(u32::MAX);
            let spouses = spouses_of(&self.gedcom, fam, Option::None);
            if spouses.len() > 1 && self.with_spouses {
                let fid = self.anim.alloc_family(FamilyNodeData::of(
                    fam,
                    false,
                    Side::None,
                    self.left_to_right,
                    generation,
                ));
                if let Some(f) = self.anim.families.get_mut(fid as usize) {
                    f.base.stamp = stamp;
                }
                for s in spouses {
                    if s.0 == person
                        && !self
                            .anim
                            .families
                            .get(fid as usize)
                            .map(|f| f.partners.contains(&pid))
                            .unwrap_or(false)
                    {
                        self.family_add_partner(fid, pid);
                    } else {
                        let q = self.make_person_node(s.0, Card::Regular, generation);
                        let same_parent = parent.and_then(|par| self.anim_parents_share(par, s.0));
                        if same_parent.unwrap_or(false) {
                            if let Some(p) = self.anim.persons.get_mut(q as usize) {
                                p.origin = parent;
                            }
                        } else {
                            self.find_acquired_ancestry(q);
                        }
                        self.check_for_duplicate(q, spouse_family);
                        self.family_add_partner(fid, q);
                    }
                }
                self.family_create_bond(fid);
                family_id = Some(fid);
            } else if let Some(p) = self.anim.persons.get_mut(pid as usize) {
                p.base.spouse_family = spouse_family;
            }
        }
        if let Some(fid) = family_id {
            self.anim.add_node(NodeId::Family(fid));
            NodeId::Family(fid)
        } else {
            self.anim.add_node(NodeId::Person(pid));
            NodeId::Person(pid)
        }
    }

    pub(crate) fn anim_parents_share(&self, parent: NodeId, spouse: u32) -> Option<bool> {
        let pfam = match parent {
            NodeId::Person(i) => self
                .anim
                .persons
                .get(i as usize)
                .and_then(|p| p.base.spouse_family),
            NodeId::Family(i) => self.anim.families.get(i as usize).map(|f| f.spouse_family),
        };
        let sfams = self
            .gedcom
            .person(spouse)
            .map(|p| p.parent_fams.clone())
            .unwrap_or_default();
        pfam.map(|f| sfams.contains(&FamilyId(f)))
    }

    pub(crate) fn family_add_partner(&mut self, fid: u32, pid: u32) {
        if let Some(f) = self.anim.families.get_mut(fid as usize) {
            if !f.partners.contains(&pid) {
                f.partners.push(pid);
            }
        }
        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
            p.family = Some(fid);
            if let Some(f) = self.anim.families.get(fid as usize) {
                p.base.spouse_family = Some(f.spouse_family);
            }
        }
    }

    pub(crate) fn family_create_bond(&mut self, fid: u32) {
        let (count, stamp) = self
            .anim
            .families
            .get(fid as usize)
            .map(|f| (f.partners.len(), f.base.stamp))
            .unwrap_or((0, Match::Main));
        if count == 1 && stamp == Match::Main {
            return;
        }
        let date = self
            .anim
            .families
            .get(fid as usize)
            .map(|f| family_marriage_date(&self.gedcom, f.spouse_family))
            .unwrap_or(Option::None);
        let mini = self
            .anim
            .families
            .get(fid as usize)
            .map(|f| f.base.mini)
            .unwrap_or(false);
        if let Some(f) = self.anim.families.get_mut(fid as usize) {
            let mut bond = BondData::empty();
            if !mini {
                bond.marriage_date = date;
            }
            f.bond = Some(bond);
        }
    }
}
