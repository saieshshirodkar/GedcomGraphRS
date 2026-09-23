use crate::config::{Branch, Card, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;

impl Graph {
    pub(crate) fn find_half_siblings(&mut self, parent_node: u32, excluded: u32, side: Side) {
        if self.with_spouses {
            return;
        }
        let person_idx = self
            .anim
            .persons
            .get(parent_node as usize)
            .map(|p| p.person)
            .unwrap_or(u32::MAX);
        let families = self
            .gedcom
            .person(person_idx)
            .map(|p| p.spouse_fams.clone())
            .unwrap_or_default();
        let mut start = 0usize;
        let mut end = families.len();
        if side == Side::Left {
            end = families.iter().position(|f| *f == excluded).unwrap_or(0);
        } else if side == Side::Right {
            start = families
                .iter()
                .position(|f| *f == excluded)
                .map(|v| v + 1)
                .unwrap_or(end);
        }
        let mut halves: Vec<u32> = Vec::new();
        for f in families[start.min(families.len())..end.min(families.len())]
            .iter()
            .copied()
        {
            if f == excluded {
                continue;
            }
            if let Some(fam) = self.gedcom.family(f) {
                halves.extend(fam.children.clone());
            }
        }
        for hs in halves {
            if self.sibling_nephew_generations > 0 {
                let fg = self.fulcrum_group;
                let genus = self.find_person_genus(
                    hs,
                    Some(NodeId::Person(parent_node)),
                    0,
                    Card::Regular,
                    Some(fg),
                );
                for n in genus.0.clone() {
                    if let NodeId::Person(i) = n {
                        if let Some(p) = self.anim.persons.get_mut(i as usize) {
                            p.half_sibling = true;
                        }
                    }
                    let limit = self.sibling_nephew_generations;
                    self.find_descendants(n, 0, limit, false);
                }
            }
        }
    }

    pub(crate) fn find_descendants(
        &mut self,
        common: NodeId,
        start_generation: i32,
        max_generations: i32,
        to_the_left: bool,
    ) {
        if self.anim.node_is_duplicate(common) {
            return;
        }
        let mut children: Vec<u32> = Vec::new();
        if let Some(sf) = self.node_spouse_family(common) {
            children.extend(
                self.gedcom
                    .family(sf)
                    .map(|f| f.children.clone())
                    .unwrap_or_default(),
            );
        } else if let NodeId::Person(i) = common {
            let person_idx = self
                .anim
                .persons
                .get(i as usize)
                .map(|p| p.person)
                .unwrap_or(u32::MAX);
            for fam in self
                .gedcom
                .person(person_idx)
                .map(|p| p.spouse_fams.clone())
                .unwrap_or_default()
            {
                children.extend(
                    self.gedcom
                        .family(fam)
                        .map(|f| f.children.clone())
                        .unwrap_or_default(),
                );
            }
        }
        if children.is_empty() {
            return;
        }
        let child_generation = self.anim.node_generation(common) + 1;
        let child_mini = child_generation >= max_generations + start_generation;
        if child_mini && !self.with_numbers {
            return;
        }
        if !child_mini && child_generation > self.max_below {
            self.max_below = child_generation;
        }
        let child_group =
            self.create_group(child_generation, child_mini, Branch::None, to_the_left);
        for child in children {
            let kind = if child_mini {
                Card::Progeny
            } else {
                Card::Regular
            };
            let genus = self.find_person_genus(
                child,
                Some(common),
                child_generation,
                kind,
                Some(child_group),
            );
            if !genus.is_empty() && !child_mini {
                for n in genus.0.clone() {
                    self.find_descendants(n, start_generation, max_generations, false);
                }
            }
        }
    }
}
