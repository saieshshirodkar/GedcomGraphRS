use crate::config::Spacing;
use crate::core::node_base::NodeId;
use crate::layout::group::Group;
use crate::layout::rows::{GroupRow, UnionRow};
use crate::layout::union::Union;
use crate::lines::duplicate::DupLine;
use crate::lines::segments::LineSeg;
use crate::nodes::family::FamilyNodeData;
use crate::nodes::person::PersonNodeData;

#[derive(Debug, Clone, Default)]
pub struct LineRowScratch {
    pub restart_x: f32,
    pub buckets: Vec<Vec<LineSeg>>,
}

#[derive(Debug, Clone, Default)]
pub struct BondView {
    pub owner: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub overlap: f32,
    pub marriage_date: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Animator {
    pub width: f32,
    pub height: f32,
    pub fulcrum_group: u32,
    pub max_above: i32,
    pub nodes: Vec<NodeId>,
    pub persons: Vec<PersonNodeData>,
    pub families: Vec<FamilyNodeData>,
    pub bond_owners: Vec<u32>,
    pub lines: Vec<LineSeg>,
    pub back_lines: Vec<LineSeg>,
    pub line_groups: Vec<Vec<LineSeg>>,
    pub back_line_groups: Vec<Vec<LineSeg>>,
    pub duplicate_lines: Vec<DupLine>,
    pub groups: Vec<Group>,
    pub group_seq: Vec<u32>,
    pub unions: Vec<Union>,
    pub group_rows: Vec<GroupRow>,
    pub union_rows: Vec<UnionRow>,
    pub left_to_right: bool,
    pub max_bitmap_size: f32,
    pub biggest_path_size: f32,
    pub spacing: Spacing,
    pub line_rows: Vec<LineRowScratch>,
    pub back_rows: Vec<LineRowScratch>,
}

impl Animator {
    pub fn fresh() -> Animator {
        Animator::default()
    }

    pub fn reset_run(&mut self) {
        self.nodes.clear();
        self.persons.clear();
        self.families.clear();
        self.bond_owners.clear();
        self.lines.clear();
        self.back_lines.clear();
        self.line_groups.clear();
        self.back_line_groups.clear();
        self.duplicate_lines.clear();
        self.groups.clear();
        self.group_seq.clear();
        self.group_rows.clear();
        self.union_rows.clear();
        self.width = 0.0;
        self.height = 0.0;
        self.biggest_path_size = 0.0;
    }

    pub(crate) fn add_node(&mut self, nid: NodeId) {
        self.nodes.push(nid);
    }

    pub(crate) fn alloc_person(&mut self, node: PersonNodeData) -> u32 {
        let id = self.persons.len() as u32;
        self.persons.push(node);
        id
    }

    pub(crate) fn alloc_family(&mut self, node: FamilyNodeData) -> u32 {
        let id = self.families.len() as u32;
        self.families.push(node);
        id
    }

    pub(crate) fn alloc_group(&mut self, group: Group) -> u32 {
        let id = self.groups.len() as u32;
        self.groups.push(group);
        id
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn person_count(&self) -> usize {
        self.persons.len()
    }

    pub fn bond_views(&self) -> Vec<BondView> {
        let mut out: Vec<BondView> = Vec::new();
        for o in &self.bond_owners {
            if let Some(f) = self.families.get(*o as usize) {
                if let Some(b) = &f.bond {
                    out.push(BondView {
                        owner: *o,
                        x: b.x,
                        y: b.y,
                        w: b.w,
                        h: b.h,
                        overlap: b.overlap,
                        marriage_date: b.marriage_date.clone(),
                    });
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Card;
    use crate::core::node_base::NodeId;
    use crate::nodes::person::PersonNodeData;

    #[test]
    fn fresh_alloc_reset() {
        let mut a = Animator::fresh();
        assert_eq!(a.node_count(), 0);
        assert_eq!(a.person_count(), 0);
        assert!(a.bond_views().is_empty());
        let id = a.alloc_person(PersonNodeData::single(0, Card::Regular, 0));
        assert_eq!(id, 0);
        assert_eq!(a.person_count(), 1);
        a.add_node(NodeId::Person(0));
        assert_eq!(a.node_count(), 1);
        a.reset_run();
        assert_eq!(a.node_count(), 0);
        assert_eq!(a.person_count(), 0);
        assert_eq!(a.width, 0.0);
    }
}
