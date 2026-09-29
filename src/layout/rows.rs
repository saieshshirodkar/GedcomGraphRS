use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

#[derive(Debug, Clone, Default)]
pub struct GroupRow {
    pub generation: i32,
    pub groups: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UnionRow {
    pub generation: i32,
    pub y_axe: f32,
    pub central: Option<NodeId>,
    pub unions: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct Genus(pub Vec<NodeId>);

impl GroupRow {
    pub fn of(generation: i32) -> GroupRow {
        GroupRow {
            generation,
            groups: Vec::new(),
        }
    }
}

impl UnionRow {
    pub fn of(generation: i32, y_axe: f32) -> UnionRow {
        UnionRow {
            generation,
            y_axe,
            central: Option::None,
            unions: Vec::new(),
        }
    }
}

impl Genus {
    pub fn empty() -> Genus {
        Genus(Vec::new())
    }

    pub fn push(&mut self, nid: NodeId) {
        self.0.push(nid);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, anim: &Animator, person: u32) -> bool {
        for nid in &self.0 {
            for p in anim.node_persons(*nid) {
                if anim
                    .persons
                    .get(p as usize)
                    .is_some_and(|q| q.person == person)
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Card;
    use crate::core::node_base::NodeId;
    use crate::nodes::person::PersonNodeData;

    #[test]
    fn genus_contains_by_gedcom_id() {
        let mut anim = Animator::fresh();
        anim.alloc_person(PersonNodeData::single(0, Card::Regular, 0));
        let mut g = Genus::empty();
        assert!(g.is_empty());
        g.push(NodeId::Person(0));
        assert_eq!(g.len(), 1);
        assert!(g.contains(&anim, 0));
        assert!(!g.contains(&anim, 1));
        assert!(!g.contains(&anim, 99));
    }
}
