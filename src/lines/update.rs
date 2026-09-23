use crate::config::Side;
use crate::core::node_base::NodeId;
use crate::lines::back_line::{BackCtx, update_back};
use crate::lines::segments::{LineKind, LineSeg};
use crate::nodes::family::FamilyNodeData;
use crate::nodes::person::PersonNodeData;

impl LineSeg {
    pub fn update(&mut self, persons: &[PersonNodeData], families: &[FamilyNodeData]) {
        match self.kind {
            LineKind::Curve { person, origin } => {
                if let Some(p) = persons.get(person as usize) {
                    let (ox, oy, oh) = origin_geom(origin, persons, families);
                    self.x1 = ox;
                    self.y1 = oy + oh;
                    self.x2 = p.center_x();
                    self.y2 = p.base.y;
                }
            }
            LineKind::Vertical { owner } => {
                if let Some(b) = families.get(owner as usize).and_then(|f| f.bond.as_ref()) {
                    let cx = b.center_x();
                    self.x1 = cx;
                    self.y1 = b.center_y();
                    self.x2 = cx;
                    self.y2 = b.y + b.h;
                }
            }
            LineKind::Horizontal { left, right, ltr } => {
                if let (Some(a), Some(b)) =
                    (persons.get(left as usize), persons.get(right as usize))
                {
                    if ltr {
                        self.x1 = a.base.x + a.base.w;
                    } else {
                        self.x1 = a.base.x;
                    }
                    self.y1 = a.center_y();
                    if ltr {
                        self.x2 = b.base.x;
                    } else {
                        self.x2 = b.base.x + b.base.w;
                    }
                    self.y2 = b.center_y();
                }
            }
            LineKind::Next {
                owner,
                partner,
                side,
                ltr,
            } => {
                let bond = families.get(owner as usize).and_then(|f| f.bond.as_ref());
                let pl = persons.get(partner as usize);
                if let (Some(b), Some(p)) = (bond, pl) {
                    self.x1 = b.center_x();
                    self.y1 = b.center_y();
                    if ltr {
                        self.x2 = if side == Side::Left { b.x } else { p.base.x };
                    } else if side == Side::Right {
                        self.x2 = b.x;
                    } else {
                        self.x2 = p.base.x;
                    }
                    self.y2 = p.center_y();
                }
            }
            LineKind::Back {
                owner,
                node,
                side,
                stamp,
                no_partners,
                ltr,
            } => {
                let ctx = BackCtx::of(owner, node, side, stamp, no_partners, ltr);
                update_back(self, ctx, persons, families);
            }
        }
    }
}

fn origin_geom(
    origin: NodeId,
    persons: &[PersonNodeData],
    families: &[FamilyNodeData],
) -> (f32, f32, f32) {
    match origin {
        NodeId::Person(i) => persons
            .get(i as usize)
            .map(|p| (p.simple_center_x(), p.base.y, p.base.h))
            .unwrap_or_default(),
        NodeId::Family(i) => families
            .get(i as usize)
            .map(|f| (f.simple_center_x(), f.base.y, f.base.h))
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Card, Match, Side};
    use crate::core::node_base::NodeId;
    use crate::nodes::family::{BondData, FamilyNodeData};
    use crate::nodes::person::PersonNodeData;

    fn couple() -> (Vec<PersonNodeData>, Vec<FamilyNodeData>) {
        let mut a = PersonNodeData::single(0, Card::Regular, 0);
        a.base.x = 100.0;
        a.base.y = 20.0;
        a.base.w = 30.0;
        a.base.h = 40.0;
        let mut b = PersonNodeData::single(1, Card::Regular, 0);
        b.base.x = 200.0;
        b.base.y = 20.0;
        b.base.w = 30.0;
        b.base.h = 40.0;
        let mut f = FamilyNodeData::of(0, false, Side::Left, true, 0);
        f.partners.push(0);
        let mut bond = BondData::empty();
        bond.x = 50.0;
        bond.y = 60.0;
        bond.w = 39.0;
        bond.h = 50.0;
        bond.overlap = 7.0;
        f.bond = Some(bond);
        (vec![a, b], vec![f])
    }

    #[test]
    fn next_left_ltr_hits_bond_edge() {
        let (p, f) = couple();
        let mut l = LineSeg::of(LineKind::Next {
            owner: 0,
            partner: 0,
            side: Side::Left,
            ltr: true,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1), (69.5, 85.0));
        assert_eq!((l.x2, l.y2), (50.0, 40.0));
    }

    #[test]
    fn next_right_rtl_hits_bond_edge() {
        let (p, f) = couple();
        let mut l = LineSeg::of(LineKind::Next {
            owner: 0,
            partner: 0,
            side: Side::Right,
            ltr: false,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1), (69.5, 85.0));
        assert_eq!((l.x2, l.y2), (50.0, 40.0));
    }

    #[test]
    fn back_middle_no_partners_uses_overlap() {
        let (p, mut f) = couple();
        f[0].base.next = Some(NodeId::Person(1));
        let mut l = LineSeg::of(LineKind::Back {
            owner: 0,
            node: 0,
            side: Side::Left,
            stamp: Match::Middle,
            no_partners: true,
            ltr: true,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1), (200.0, 85.0));
        assert_eq!((l.x2, l.y2), (57.0, 85.0));
    }

    #[test]
    fn horizontal_rtl_uses_outer_edges() {
        let (p, f) = couple();
        let mut l = LineSeg::of(LineKind::Horizontal {
            left: 0,
            right: 1,
            ltr: false,
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1, l.x2, l.y2), (100.0, 40.0, 230.0, 40.0));
    }

    #[test]
    fn curve_from_family_origin() {
        let (p, mut f) = couple();
        f[0].base.y = 10.0;
        f[0].base.h = 40.0;
        let mut l = LineSeg::of(LineKind::Curve {
            person: 0,
            origin: NodeId::Family(0),
        });
        l.update(&p, &f);
        assert_eq!((l.x1, l.y1), (69.5, 50.0));
        assert_eq!((l.x2, l.y2), (115.0, 20.0));
    }
}
