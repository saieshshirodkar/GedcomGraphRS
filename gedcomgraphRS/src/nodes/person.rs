use crate::config::Card;
use crate::core::node_base::{NodeBase, NodeId};

#[derive(Debug, Clone)]
pub struct PersonNodeData {
    pub base: NodeBase,
    pub person: u32,
    pub origin: Option<NodeId>,
    pub family: Option<u32>,
    pub kind: Card,
    pub acquired: bool,
    pub dead: bool,
    pub amount: u32,
    pub duplicate: bool,
    pub half_sibling: bool,
}

impl PersonNodeData {
    pub fn single(person: u32, kind: Card, generation: i32) -> PersonNodeData {
        PersonNodeData {
            base: NodeBase::of_generation(generation),
            person,
            origin: Option::None,
            family: Option::None,
            kind,
            acquired: false,
            dead: false,
            amount: 0,
            duplicate: false,
            half_sibling: false,
        }
    }

    pub fn is_fulcrum(&self) -> bool {
        self.kind == Card::Fulcrum
    }

    pub fn is_mini_card(&self) -> bool {
        self.kind == Card::Ancestry || self.kind == Card::Progeny
    }

    pub fn center_rel_x(&self) -> f32 {
        self.base.w / 2.0
    }

    pub fn center_rel_y(&self) -> f32 {
        self.base.h / 2.0
    }

    pub fn center_x(&self) -> f32 {
        self.base.x + self.center_rel_x()
    }

    pub fn center_y(&self) -> f32 {
        self.base.y + self.center_rel_y()
    }

    pub fn simple_center_x(&self) -> f32 {
        self.base.x + self.base.w / 2.0
    }

    pub fn apply_x(&mut self, x: f32) {
        self.base.force += x - self.base.x;
        self.base.x = x;
    }

    pub fn apply_y(&mut self, y: f32) {
        self.base.y = y;
    }

    pub fn label(&self, essence: &str) -> String {
        if self.base.mini {
            format!("{} ({essence})", self.amount)
        } else {
            let mut txt = format!(" {essence}");
            if self.duplicate {
                txt.push_str(" (2)");
            }
            txt.trim().to_string()
        }
    }

    pub fn main_self(&self) -> bool {
        !self.half_sibling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fulcrum_flag() {
        let p = PersonNodeData::single(4, Card::Fulcrum, 0);
        assert!(p.is_fulcrum());
        assert!(!p.is_mini_card());
        let q = PersonNodeData::single(4, Card::Ancestry, -2);
        assert!(q.is_mini_card());
        assert!(!q.is_fulcrum());
    }

    #[test]
    fn centers() {
        let mut p = PersonNodeData::single(1, Card::Regular, 0);
        p.base.x = 10.0;
        p.base.y = 20.0;
        p.base.w = 30.0;
        p.base.h = 40.0;
        assert_eq!(p.center_x(), 25.0);
        assert_eq!(p.center_y(), 40.0);
        assert_eq!(p.simple_center_x(), 25.0);
        assert_eq!(p.center_rel_x(), 15.0);
        assert_eq!(p.center_rel_y(), 20.0);
    }

    #[test]
    fn apply_tracks_force() {
        let mut p = PersonNodeData::single(1, Card::Regular, 0);
        p.apply_x(12.0);
        assert_eq!(p.base.x, 12.0);
        assert_eq!(p.base.force, 12.0);
        p.apply_y(9.0);
        assert_eq!(p.base.y, 9.0);
    }

    #[test]
    fn labels() {
        let mut p = PersonNodeData::single(1, Card::Regular, 0);
        assert_eq!(p.label("Ann"), "Ann");
        p.duplicate = true;
        assert_eq!(p.label("Ann"), "Ann (2)");
        p.base.mini = true;
        p.amount = 7;
        assert_eq!(p.label("Ann"), "7 (Ann)");
        assert!(p.main_self());
        p.half_sibling = true;
        assert!(!p.main_self());
    }
}
