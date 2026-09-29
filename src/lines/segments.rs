use crate::config::{Match, Side};
use crate::core::node_base::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Curve {
        person: u32,
        origin: NodeId,
    },
    Vertical {
        owner: u32,
    },
    Horizontal {
        left: u32,
        right: u32,
        ltr: bool,
    },
    Next {
        owner: u32,
        partner: u32,
        side: Side,
        ltr: bool,
    },
    Back {
        owner: u32,
        node: u32,
        side: Side,
        stamp: Match,
        no_partners: bool,
        ltr: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSeg {
    pub kind: LineKind,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl LineSeg {
    pub fn of(kind: LineKind) -> LineSeg {
        LineSeg {
            kind,
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        }
    }

    pub fn left(&self) -> f32 {
        self.x1.min(self.x2)
    }

    pub fn right(&self) -> f32 {
        self.x1.max(self.x2)
    }

    pub fn span(&self) -> f32 {
        self.right() - self.left()
    }

    pub fn compare(&self, other: &LineSeg) -> std::cmp::Ordering {
        debug_assert!(self.left().is_finite() && other.left().is_finite());
        self.left()
            .partial_cmp(&other.left())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Card;
    use crate::nodes::family::FamilyNodeData;
    use crate::nodes::person::PersonNodeData;

    fn persons() -> Vec<PersonNodeData> {
        let mut a = PersonNodeData::single(0, Card::Regular, 0);
        a.base.x = 10.0;
        a.base.y = 20.0;
        a.base.w = 30.0;
        a.base.h = 40.0;
        let mut b = PersonNodeData::single(1, Card::Regular, 0);
        b.base.x = 100.0;
        b.base.y = 20.0;
        b.base.w = 30.0;
        b.base.h = 40.0;
        vec![a, b]
    }

    #[test]
    fn curve_update() {
        let p = persons();
        let f: Vec<FamilyNodeData> = Vec::new();
        let mut l = LineSeg::of(LineKind::Curve {
            person: 1,
            origin: NodeId::Person(0),
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1), (25.0, 60.0));
        assert_eq!((l.x2, l.y2), (115.0, 20.0));
    }

    #[test]
    fn horizontal_update() {
        let p = persons();
        let f: Vec<FamilyNodeData> = Vec::new();
        let mut l = LineSeg::of(LineKind::Horizontal {
            left: 0,
            right: 1,
            ltr: true,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1, l.x2, l.y2), (40.0, 40.0, 100.0, 40.0));
        let mut r = LineSeg::of(LineKind::Horizontal {
            left: 0,
            right: 1,
            ltr: false,
        });
        r.update(&p, &f);
        assert_eq!((r.x1, r.x2), (10.0, 130.0));
    }

    #[test]
    fn ordering_and_span() {
        let a = LineSeg {
            kind: LineKind::Vertical { owner: 0 },
            x1: 5.0,
            y1: 0.0,
            x2: 9.0,
            y2: 0.0,
        };
        let b = LineSeg {
            kind: LineKind::Vertical { owner: 1 },
            x1: 1.0,
            y1: 0.0,
            x2: 2.0,
            y2: 0.0,
        };
        assert_eq!(a.compare(&b), std::cmp::Ordering::Greater);
        assert_eq!(a.left(), 5.0);
        assert_eq!(a.span(), 4.0);
    }

    #[test]
    fn back_missing_bond_keeps_coords() {
        let p = persons();
        let f: Vec<FamilyNodeData> = Vec::new();
        let mut l = LineSeg::of(LineKind::Back {
            owner: 0,
            node: 0,
            side: Side::Left,
            stamp: Match::Main,
            no_partners: false,
            ltr: true,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1, l.x2, l.y2), (0.0, 0.0, 0.0, 0.0));
    }
}
