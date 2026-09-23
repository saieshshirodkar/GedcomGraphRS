use crate::config::{Match, Side};
use crate::core::node_base::NodeId;
use crate::lines::segments::LineSeg;
use crate::nodes::family::FamilyNodeData;
use crate::nodes::person::PersonNodeData;

fn node_edge_x(nid: NodeId, persons: &[PersonNodeData], families: &[FamilyNodeData]) -> (f32, f32) {
    match nid {
        NodeId::Person(i) => persons
            .get(i as usize)
            .map(|p| (p.base.x, p.base.w))
            .unwrap_or_default(),
        NodeId::Family(i) => families
            .get(i as usize)
            .map(|f| (f.base.x, f.base.w))
            .unwrap_or_default(),
    }
}

pub(crate) struct BackCtx {
    pub owner: u32,
    pub node: u32,
    pub side: Side,
    pub stamp: Match,
    pub no_partners: bool,
    pub ltr: bool,
}

impl BackCtx {
    pub fn of(
        owner: u32,
        node: u32,
        side: Side,
        stamp: Match,
        no_partners: bool,
        ltr: bool,
    ) -> BackCtx {
        BackCtx {
            owner,
            node,
            side,
            stamp,
            no_partners,
            ltr,
        }
    }
}

pub(crate) fn update_back(
    line: &mut LineSeg,
    ctx: BackCtx,
    persons: &[PersonNodeData],
    families: &[FamilyNodeData],
) {
    let fam = families.get(ctx.owner as usize);
    let node_fam = families.get(ctx.node as usize);
    let (bond, prev, next) = match (fam, node_fam) {
        (Some(f), Some(n)) => (f.bond.as_ref(), n.base.prev, n.base.next),
        _ => return,
    };
    let Some(b) = bond else { return };
    if ctx.ltr {
        if ctx.side == Side::Left {
            if let Some(n) = next {
                let (nx, _) = node_edge_x(n, persons, families);
                line.x1 = nx;
            }
        } else if let Some(p) = prev {
            let (px, pw) = node_edge_x(p, persons, families);
            line.x1 = px + pw;
        }
    } else if ctx.side == Side::Right {
        if let Some(p) = prev {
            let (px, _) = node_edge_x(p, persons, families);
            line.x1 = px;
        }
    } else if let Some(n) = next {
        let (nx, nw) = node_edge_x(n, persons, families);
        line.x1 = nx + nw;
    }
    line.y1 = b.center_y();
    if ctx.ltr {
        if b.marriage_date.is_some() {
            line.x2 = if ctx.side == Side::Left {
                b.x + b.w
            } else {
                b.x
            };
        } else if ctx.no_partners && ctx.stamp == Match::Middle {
            if ctx.side == Side::Left {
                line.x2 = b.x + b.overlap;
            } else {
                line.x2 = b.x + b.w - b.overlap;
            }
        } else {
            line.x2 = b.center_x();
        }
    } else if b.marriage_date.is_some() {
        line.x2 = if ctx.side == Side::Right {
            b.x + b.w
        } else {
            b.x
        };
    } else if ctx.no_partners && ctx.stamp == Match::Middle {
        if ctx.side == Side::Right {
            line.x2 = b.x + b.overlap;
        } else {
            line.x2 = b.x + b.w - b.overlap;
        }
    } else {
        line.x2 = b.center_x();
    }
    line.y2 = b.center_y();
}
