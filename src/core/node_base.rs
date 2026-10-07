use crate::config::Match;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    Person(u32),
    Family(u32),
}

#[derive(Debug, Clone, Default)]
pub struct NodeBase {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub generation: i32,
    pub mini: bool,
    pub is_ancestor: bool,
    pub married_siblings: bool,
    pub union: Option<u32>,
    pub group: Option<u32>,
    pub youth: Option<u32>,
    pub prev: Option<NodeId>,
    pub next: Option<NodeId>,
    pub stamp: Match,
    pub origins: Option<Vec<NodeId>>,
    pub force: f32,
    pub column_shift: f32,
    pub spouse_family: Option<u32>,
}

impl NodeBase {
    pub fn of_generation(generation: i32) -> NodeBase {
        NodeBase {
            generation,
            ..NodeBase::default()
        }
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.center_rel_simple()
    }

    pub fn center_rel_simple(&self) -> f32 {
        self.w / 2.0
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn is_multi_marriage(&self) -> bool {
        self.stamp.is_multi()
    }

    pub fn shift(&mut self, dx: f32) {
        self.x += dx;
    }

    pub fn has_finite_rect(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.w.is_finite() && self.h.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_geometry() {
        let mut b = NodeBase::of_generation(-2);
        b.x = 10.0;
        b.w = 20.0;
        assert_eq!(b.center_x(), 20.0);
        assert_eq!(b.right(), 30.0);
        b.shift(5.0);
        assert_eq!(b.x, 15.0);
        assert!(b.has_finite_rect());
    }

    #[test]
    fn multi_marriage_flag() {
        let mut b = NodeBase::default();
        assert!(!b.is_multi_marriage());
        b.stamp = Match::Near;
        assert!(b.is_multi_marriage());
        assert_eq!(b.bottom(), b.y + b.h);
    }
}
