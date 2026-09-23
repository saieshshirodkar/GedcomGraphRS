use crate::config::{Card, Match, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;
use crate::layout::rows::Genus;
use crate::model::helpers::person_is_female;

impl Graph {
    pub(crate) fn find_ancestor_genus(&mut self, person_node: u32, group: u32, side: Side) {
        let (kind, duplicate, family) = self
            .anim
            .persons
            .get(person_node as usize)
            .map(|p| (p.kind, p.duplicate, p.family))
            .unwrap_or((Card::Regular, true, Option::None));
        let Some(famid) = family else { return };
        let mut genus = Genus::empty();
        genus.push(NodeId::Family(famid));
        if kind != Card::Regular || !self.with_spouses || duplicate {
            return;
        }
        let person_idx = self
            .anim
            .persons
            .get(person_node as usize)
            .map(|p| p.person)
            .unwrap_or(u32::MAX);
        let mut families = self
            .gedcom
            .person(person_idx)
            .map(|p| p.spouse_fams.clone())
            .unwrap_or_default();
        if families.len() <= 1 {
            return;
        }
        let main_fam = self
            .anim
            .families
            .get(famid as usize)
            .map(|f| f.spouse_family)
            .unwrap_or(u32::MAX);
        families.retain(|f| *f != main_fam);
        let generation = self
            .anim
            .persons
            .get(person_node as usize)
            .map(|p| p.base.generation)
            .unwrap_or(0);
        let origin = self
            .anim
            .persons
            .get(person_node as usize)
            .and_then(|p| p.origin);
        let total = families.len();
        for (i, next_family) in families.clone().into_iter().enumerate() {
            let stamp = Match::get_for_ancestors(total, i, side);
            let next = self.create_next_family_node(
                next_family,
                person_idx,
                generation,
                side,
                stamp,
                origin,
            );
            if side == Side::Left {
                let at = self
                    .anim
                    .groups
                    .get(group as usize)
                    .and_then(|g| g.list.iter().position(|n| *n == NodeId::Family(famid)));
                self.anim.group_add_node(group, NodeId::Family(next), at);
                if let Some(pos) = genus.0.iter().position(|n| *n == NodeId::Family(famid)) {
                    genus.0.insert(pos, NodeId::Family(next));
                }
            } else {
                let base = self
                    .anim
                    .groups
                    .get(group as usize)
                    .and_then(|g| g.list.iter().position(|n| *n == NodeId::Family(famid)))
                    .unwrap_or(0);
                let at = base + genus.len();
                let len = self
                    .anim
                    .groups
                    .get(group as usize)
                    .map(|g| g.list.len())
                    .unwrap_or(0);
                self.anim
                    .group_add_node(group, NodeId::Family(next), Some(at.min(len)));
                genus.push(NodeId::Family(next));
            }
        }
        for node in genus.0.clone() {
            if node != NodeId::Family(famid) {
                if generation < -1 {
                    if self.with_numbers && -generation <= self.great_uncles_generations + 1 {
                        self.find_descendants(node, generation, 0, false);
                    }
                } else if self.sibling_nephew_generations > 0 {
                    let limit = self.sibling_nephew_generations + 1;
                    self.find_descendants(node, -1, limit, side == Side::Left);
                }
            }
        }
    }
    pub(crate) fn find_person_genus(
        &mut self,
        person: u32,
        parent: Option<NodeId>,
        generation: i32,
        kind: Card,
        group: Option<u32>,
    ) -> Genus {
        let mut genus = Genus::empty();
        if let Some(g) = group {
            if self.group_contains_person(g, person) {
                return genus;
            }
        }
        let families = self
            .gedcom
            .person(person)
            .map(|p| p.spouse_fams.clone())
            .unwrap_or_default();
        if families.is_empty() || !self.with_spouses || kind == Card::Progeny {
            let single = self.create_node_from_person(
                person,
                Option::None,
                parent,
                generation,
                kind,
                Match::Main,
            );
            if let Some(g) = group {
                self.anim.group_add_node(g, single, Option::None);
            }
            genus.push(single);
            return genus;
        }
        let female = person_is_female(&self.gedcom, person);
        let side = if female { Side::Right } else { Side::Left };
        let mut straight = true;
        if families.len() > 1 {
            if side == Side::Left {
                let last = families[families.len() - 1];
                let spouses = self.get_spouses(last, Option::None);
                if spouses.get(1).copied() == Some(person) {
                    straight = false;
                }
            } else {
                let spouses = self.get_spouses(families[0], Option::None);
                if spouses.first().copied() == Some(person) {
                    straight = false;
                }
            }
        }
        for (i, family) in families.clone().into_iter().enumerate() {
            let stamp = Match::get(families.len(), i, side, straight);
            let node = if stamp == Match::Main {
                self.create_node_from_person(person, Some(family), parent, generation, kind, stamp)
            } else {
                NodeId::Family(
                    self.create_next_family_node(family, person, generation, side, stamp, parent),
                )
            };
            if let Some(g) = group {
                self.anim.group_add_node(g, node, Option::None);
            }
            genus.push(node);
        }
        genus
    }
}
