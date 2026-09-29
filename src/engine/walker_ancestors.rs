use crate::config::{Branch, Card, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;
use crate::model::helpers::person_is_female;

impl Graph {
    pub(crate) fn find_ancestors(
        &mut self,
        common: u32,
        group: u32,
        generation_up: i32,
        sibling_partner: bool,
    ) {
        let family = self
            .anim
            .persons
            .get(common as usize)
            .and_then(|p| p.family);
        if sibling_partner {
            if let Some(f) = family {
                let partners = self
                    .anim
                    .families
                    .get(f as usize)
                    .map(|x| x.partners.clone())
                    .unwrap_or_default();
                if partners.get(1).copied() == Some(common) {
                    let first_origin = partners
                        .first()
                        .and_then(|q| self.anim.persons.get(*q as usize))
                        .and_then(|q| q.origin);
                    self.set_person_origin(common, first_origin);
                    return;
                }
            }
        }
        if self
            .anim
            .persons
            .get(common as usize)
            .is_some_and(|p| p.duplicate)
        {
            return;
        }
        let person_idx = self
            .anim
            .persons
            .get(common as usize)
            .map(|p| p.person)
            .unwrap_or(u32::MAX);
        let parents = self
            .gedcom
            .person(person_idx)
            .map(|p| p.parent_fams.clone())
            .unwrap_or_default();
        if parents.is_empty() {
            return;
        }
        let family_ged = parents[parents.len() - 1].0;
        let parent_gen = generation_up + 1;
        let parent_mini = parent_gen > self.ancestor_generations;
        let first_group = self.create_group(-parent_gen, parent_mini, Branch::None, false);
        let kind = if parent_mini {
            Card::Ancestry
        } else {
            Card::Regular
        };
        let parent_node = self.create_node_from_family(family_ged, -parent_gen, kind);
        self.set_person_origin(common, Some(NodeId::Family(parent_node)));
        if generation_up > 1 {
            let multi = family.is_some_and(|f| {
                self.anim
                    .families
                    .get(f as usize)
                    .map(|x| x.partners.len() > 1)
                    .unwrap_or(false)
            });
            if multi && !sibling_partner {
                self.find_uncles(common, group, Side::None);
            } else {
                self.find_uncles(common, group, Side::Left);
                self.find_uncles(common, group, Side::Right);
            }
        }
        let parent_size = self.anim.node_person_count(NodeId::Family(parent_node));
        if parent_size == 0 {
            return;
        }
        self.set_node_ancestor(NodeId::Family(parent_node), true);
        let first = self.anim.node_partner(NodeId::Family(parent_node), 0);
        let second = if parent_size > 1 {
            self.anim.node_partner(NodeId::Family(parent_node), 1)
        } else {
            Option::None
        };
        let sibling_parents = self.are_siblings(first, second);
        let branch = if parent_size > 1 && !sibling_parents {
            Branch::Pater
        } else {
            Branch::None
        };
        self.set_group_branch(first_group, branch);
        self.anim
            .group_add_node(first_group, NodeId::Family(parent_node), Option::None);
        if parent_gen > self.max_above && !parent_mini {
            self.max_above = parent_gen;
        }
        if generation_up < self.ancestor_generations {
            if let Some(s) = second {
                if let Some(f) = first {
                    self.find_ancestors(f, first_group, parent_gen, sibling_parents);
                    self.find_ancestor_genus(f, first_group, Side::Left);
                    if sibling_parents {
                        self.find_ancestors(s, first_group, parent_gen, true);
                        self.find_ancestor_genus(s, first_group, Side::Right);
                    } else {
                        let g2 = self.create_group(-parent_gen, parent_mini, Branch::Mater, false);
                        self.anim
                            .group_add_node(g2, NodeId::Family(parent_node), Option::None);
                        self.find_ancestors(s, g2, parent_gen, false);
                        self.find_ancestor_genus(s, g2, Side::Right);
                    }
                }
            } else if let Some(f) = first {
                self.find_ancestors(f, first_group, parent_gen, false);
                let female = person_is_female(
                    &self.gedcom,
                    self.anim
                        .persons
                        .get(f as usize)
                        .map(|p| p.person)
                        .unwrap_or(u32::MAX),
                );
                self.find_ancestor_genus(
                    f,
                    first_group,
                    if female { Side::Right } else { Side::Left },
                );
            }
        }
    }
}
