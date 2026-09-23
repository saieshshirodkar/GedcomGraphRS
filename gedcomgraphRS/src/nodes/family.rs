use crate::config::{BOND_WIDTH, MARRIAGE_INNER_WIDTH, MINI_BOND_WIDTH, Side};
use crate::core::node_base::NodeBase;
use std::fmt::Write as _;

#[derive(Debug, Clone, Default)]
pub struct BondData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub marriage_date: Option<String>,
    pub overlap: f32,
}

#[derive(Debug, Clone)]
pub struct FamilyNodeData {
    pub base: NodeBase,
    pub spouse_family: u32,
    pub partners: Vec<u32>,
    pub bond: Option<BondData>,
    pub side: Side,
    pub left_to_right: bool,
}

impl BondData {
    pub fn empty() -> BondData {
        BondData::default()
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.w / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.y + self.h / 2.0
    }

    pub fn place_x(&mut self, x: f32) {
        self.x = x - self.overlap;
    }

    pub fn marriage_year(&self) -> String {
        if let Some(d) = &self.marriage_date {
            if let Some(i) = d.rfind(' ') {
                if i > 0 {
                    return d[i..].to_string();
                }
            }
            return d.clone();
        }
        String::new()
    }

    pub fn label(&self) -> String {
        let t = self.marriage_year();
        if t.is_empty() { "-•-".to_string() } else { t }
    }
}

impl FamilyNodeData {
    pub fn of(
        family: u32,
        mini: bool,
        side: Side,
        left_to_right: bool,
        generation: i32,
    ) -> FamilyNodeData {
        let mut base = NodeBase::of_generation(generation);
        base.mini = mini;
        FamilyNodeData {
            base,
            spouse_family: family,
            partners: Vec::new(),
            bond: Option::None,
            side,
            left_to_right,
        }
    }

    pub fn bond_width(&self) -> f32 {
        if let Some(b) = &self.bond {
            if self.base.mini {
                MINI_BOND_WIDTH
            } else if b.marriage_date.is_some() {
                MARRIAGE_INNER_WIDTH
            } else {
                BOND_WIDTH
            }
        } else {
            0.0
        }
    }

    pub fn center_rel_y(&self) -> f32 {
        self.base.h / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.base.y + self.center_rel_y()
    }

    pub fn simple_center_x(&self) -> f32 {
        if let Some(b) = &self.bond {
            b.x + b.w / 2.0
        } else {
            self.base.x + self.base.w / 2.0
        }
    }

    pub fn has_children_flag(&self) -> bool {
        if self.base.mini {
            true
        } else {
            self.base.youth.is_some()
        }
    }

    pub fn describe(&self, parts: &[String]) -> String {
        let mut s = String::from("{");
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let _ = write!(s, "{p}");
        }
        s.push('}');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MARRIAGE_WIDTH;

    #[test]
    fn bond_centers() {
        let mut b = BondData::empty();
        b.x = 10.0;
        b.w = 20.0;
        b.y = 4.0;
        b.h = 8.0;
        assert_eq!(b.center_x(), 20.0);
        assert_eq!(b.center_y(), 8.0);
        b.overlap = 3.0;
        b.place_x(50.0);
        assert_eq!(b.x, 47.0);
    }

    #[test]
    fn marriage_year_tail() {
        let mut b = BondData::empty();
        assert_eq!(b.label(), "-•-");
        b.marriage_date = Some("12 MAY 1895".to_string());
        assert_eq!(b.marriage_year(), " 1895");
        assert_eq!(b.label(), " 1895");
        b.marriage_date = Some("1895".to_string());
        assert_eq!(b.marriage_year(), "1895");
    }

    #[test]
    fn bond_width_rules() {
        let mut f = FamilyNodeData::of(0, false, Side::None, false, 0);
        assert_eq!(f.bond_width(), 0.0);
        f.bond = Some(BondData::empty());
        assert_eq!(f.bond_width(), BOND_WIDTH);
        if let Some(b) = f.bond.as_mut() {
            b.marriage_date = Some("1900".to_string());
        }
        assert_eq!(f.bond_width(), MARRIAGE_INNER_WIDTH);
        let mut m = FamilyNodeData::of(0, true, Side::None, false, 0);
        m.bond = Some(BondData::empty());
        assert_eq!(m.bond_width(), MINI_BOND_WIDTH);
        assert_eq!(f.center_rel_y(), f.base.h / 2.0);
    }

    #[test]
    fn children_and_centers() {
        let f = FamilyNodeData::of(2, false, Side::Left, true, 1);
        assert!(!f.has_children_flag());
        let m = FamilyNodeData::of(2, true, Side::None, false, 1);
        assert!(m.has_children_flag());
        assert_eq!(f.simple_center_x(), f.base.x + f.base.w / 2.0);
        assert_eq!(f.describe(&["a".to_string(), "b".to_string()]), "{a, b}");
        assert_eq!(
            FamilyNodeData::of(0, false, Side::None, false, 0).describe(&[]),
            "{}"
        );
        assert_eq!(MARRIAGE_WIDTH, 39.0);
    }
}
