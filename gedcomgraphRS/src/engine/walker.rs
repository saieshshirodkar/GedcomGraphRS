use crate::config::{Branch, Card, Side};
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;

impl Graph {
    pub fn walk(&mut self, fulcrum: u32) {
        self.fulcrum = Some(fulcrum);
        self.anim.reset_run();
        self.max_above = 0;
        self.max_below = 0;
        let parent_families = self
            .gedcom
            .person(fulcrum)
            .map(|p| p.parent_fams.clone())
            .unwrap_or_default();
        let fg = self.create_group(0, false, Branch::None, false);
        self.fulcrum_group = fg;
        if !parent_families.is_empty() {
            if self.which_family >= parent_families.len() {
                self.which_family = parent_families.len() - 1;
            }
            let parent_family = parent_families[self.which_family];
            let parent_mini = self.ancestor_generations == 0;
            let parent_kind = if parent_mini {
                Card::Ancestry
            } else {
                Card::Regular
            };
            let parent_node = self.create_node_from_family(parent_family, -1, parent_kind);
            let parent_size = self.anim.node_person_count(NodeId::Family(parent_node));
            let first = self.anim.node_partner(NodeId::Family(parent_node), 0);
            let second = self.anim.node_partner(NodeId::Family(parent_node), 1);
            let parent_siblings = self.are_siblings(first, second);
            let mut first_parent_group: Option<u32> = Option::None;
            if parent_size > 0 && self.ancestor_generations > 0 {
                self.set_node_ancestor(NodeId::Family(parent_node), true);
                if parent_siblings {
                    if let Some(s) = second {
                        self.set_person_origin(s, Some(NodeId::Family(parent_node)));
                    }
                }
                let g = self.create_group(-1, parent_mini, Branch::None, false);
                let branch = if parent_size > 1 && !parent_siblings {
                    Branch::Pater
                } else {
                    Branch::None
                };
                self.set_group_branch(g, branch);
                self.anim
                    .group_add_node(g, NodeId::Family(parent_node), Option::None);
                if !parent_mini {
                    self.max_above = 1;
                }
                if let Some(f) = first {
                    self.find_ancestors(f, g, 1, parent_siblings);
                    let side = if parent_size == 1 || parent_siblings {
                        Side::Left
                    } else {
                        Side::None
                    };
                    self.find_uncles(f, g, side);
                    self.find_ancestor_genus(f, g, Side::Left);
                    let hs = if parent_size == 1 {
                        Side::Left
                    } else {
                        Side::None
                    };
                    self.find_half_siblings(f, parent_family, hs);
                }
                first_parent_group = Some(g);
            }
            let fulcrum_genus = self.find_person_genus(
                fulcrum,
                Some(NodeId::Family(parent_node)),
                0,
                Card::Fulcrum,
                Option::None,
            );
            let siblings = self
                .gedcom
                .family(parent_family)
                .map(|f| f.children.clone())
                .unwrap_or_default();
            for sib in siblings {
                if sib == fulcrum {
                    for node in fulcrum_genus.0.clone() {
                        self.anim.group_add_node(fg, node, Option::None);
                        let limit = self.descendant_generations + 1;
                        self.find_descendants(node, 0, limit, false);
                    }
                } else if self.sibling_nephew_generations > 0
                    && !fulcrum_genus.contains(&self.anim, &self.gedcom, sib)
                {
                    let genus = self.find_person_genus(
                        sib,
                        Some(NodeId::Family(parent_node)),
                        0,
                        Card::Regular,
                        Some(fg),
                    );
                    for n in genus.0.clone() {
                        let limit = self.sibling_nephew_generations;
                        self.find_descendants(n, 0, limit, false);
                    }
                }
            }
            if parent_size > 0 && self.ancestor_generations > 0 {
                if second.is_none() {
                    if let (Some(f), Some(g)) = (first, first_parent_group) {
                        self.find_half_siblings(f, parent_family, Side::Right);
                        self.find_uncles(f, g, Side::Right);
                    }
                } else if parent_siblings {
                    if let (Some(f), Some(s), Some(g)) = (first, second, first_parent_group) {
                        self.find_half_siblings(s, parent_family, Side::None);
                        self.find_ancestors(s, g, 1, true);
                        self.find_ancestor_genus(s, g, Side::Right);
                        self.find_uncles(f, g, Side::Right);
                    }
                } else if let (Some(s), Some(g0)) = (second, first_parent_group) {
                    let g2 = self.create_group(-1, parent_mini, Branch::Mater, false);
                    self.anim
                        .group_add_node(g2, NodeId::Family(parent_node), Option::None);
                    self.find_half_siblings(s, parent_family, Side::None);
                    self.find_ancestors(s, g2, 1, false);
                    self.find_ancestor_genus(s, g2, Side::Right);
                    self.find_uncles(s, g2, Side::None);
                    let _ = g0;
                }
            }
        } else {
            let genus = self.find_person_genus(fulcrum, Option::None, 0, Card::Fulcrum, Some(fg));
            for node in genus.0.clone() {
                let limit = self.descendant_generations + 1;
                self.find_descendants(node, 0, limit, false);
            }
        }
    }

    pub(crate) fn set_node_ancestor(&mut self, nid: NodeId, value: bool) {
        match nid {
            NodeId::Person(i) => {
                if let Some(p) = self.anim.persons.get_mut(i as usize) {
                    p.base.is_ancestor = value;
                }
            }
            NodeId::Family(i) => {
                if let Some(f) = self.anim.families.get_mut(i as usize) {
                    f.base.is_ancestor = value;
                }
            }
        }
    }

    pub(crate) fn set_group_branch(&mut self, gid: u32, branch: Branch) {
        if let Some(g) = self.anim.groups.get_mut(gid as usize) {
            g.branch = branch;
        }
    }

    pub(crate) fn set_person_origin(&mut self, pid: u32, origin: Option<NodeId>) {
        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
            p.origin = origin;
        }
    }
}
