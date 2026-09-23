use crate::config::{Branch, Card, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;

impl Graph {
    pub(crate) fn find_uncles(&mut self, person_node: u32, group: u32, side: Side) {
        let generation = self
            .anim
            .persons
            .get(person_node as usize)
            .map(|p| p.base.generation)
            .unwrap_or(0);
        let generation_up = -generation;
        let deep_enough = generation_up <= self.great_uncles_generations;
        let cousins = generation_up == 1 && self.uncle_cousin_generations > 0;
        if !deep_enough && !cousins {
            return;
        }
        let origin = self
            .anim
            .persons
            .get(person_node as usize)
            .and_then(|p| p.origin);
        let Some(org) = origin else { return };
        let branch = self
            .anim
            .groups
            .get(group as usize)
            .map(|g| g.branch)
            .unwrap_or(Branch::None);
        let person_idx = self
            .anim
            .persons
            .get(person_node as usize)
            .map(|p| p.person)
            .unwrap_or(u32::MAX);
        let spouse_fam = self.node_spouse_family(org);
        let Some(sf) = spouse_fam else { return };
        let uncles = self
            .gedcom
            .family(sf)
            .map(|f| f.children.clone())
            .unwrap_or_default();
        let mut start = 0usize;
        let mut end = uncles.len();
        if branch == Branch::None {
            if side == Side::Left {
                if let Some(pos) = uncles.iter().position(|u| *u == person_idx) {
                    end = pos;
                }
            } else if side == Side::Right {
                if let Some(pos) = uncles.iter().position(|u| *u == person_idx) {
                    start = pos + 1;
                } else {
                    start = end;
                }
            }
        }
        let mut position = 0usize;
        for uncle in uncles.iter().take(end.min(uncles.len())).skip(start) {
            let uncle = *uncle;
            if self.group_contains_person(group, uncle) {
                continue;
            }
            let genus = self.find_person_genus(
                uncle,
                Some(org),
                -generation_up,
                Card::Regular,
                Option::None,
            );
            for node in genus.0.clone() {
                if branch == Branch::Pater || (branch == Branch::None && side == Side::Left) {
                    let len = self
                        .anim
                        .groups
                        .get(group as usize)
                        .map(|g| g.list.len())
                        .unwrap_or(0);
                    self.anim
                        .group_add_node(group, node, Some(position.min(len)));
                    position += 1;
                } else {
                    self.anim.group_add_node(group, node, Option::None);
                }
                if generation_up == 1 {
                    let left = position > 0;
                    self.find_descendants(node, -1, self.uncle_cousin_generations, left);
                } else {
                    self.find_descendants(node, -generation_up, 1, false);
                }
            }
        }
    }

    pub(crate) fn node_spouse_family(&self, nid: NodeId) -> Option<u32> {
        match nid {
            NodeId::Person(i) => self
                .anim
                .persons
                .get(i as usize)
                .and_then(|p| p.base.spouse_family),
            NodeId::Family(i) => self.anim.families.get(i as usize).map(|f| f.spouse_family),
        }
    }

    pub(crate) fn group_contains_person(&self, gid: u32, person: u32) -> bool {
        let target = self
            .gedcom
            .person(person)
            .map(|p| p.id.clone())
            .unwrap_or_default();
        if let Some(g) = self.anim.groups.get(gid as usize) {
            for nid in g.list.clone() {
                for p in self.anim.node_persons(nid) {
                    if let Some(q) = self.anim.persons.get(p as usize) {
                        if let Some(gp) = self.gedcom.person(q.person) {
                            if gp.id == target {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
