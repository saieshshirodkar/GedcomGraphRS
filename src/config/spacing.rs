pub const VERTICAL_SPACE: f32 = 90.0;
pub const HORIZONTAL_SPACE: f32 = 15.0;
pub const UNION_DISTANCE: f32 = 35.0;

pub const BOND_WIDTH: f32 = 23.0;
pub const MINI_BOND_WIDTH: f32 = 18.0;
pub const MARRIAGE_WIDTH: f32 = 39.0;
pub const MARRIAGE_INNER_WIDTH: f32 = 25.0;
pub const MARRIAGE_HEIGHT: f32 = 25.0;
pub const HEARTH_DIAMETER: f32 = 8.0;
pub const MINI_HEARTH_DIAMETER: f32 = 6.0;

pub const LITTLE_GROUP_DISTANCE: f32 = 60.0;
pub const ANCESTRY_DISTANCE: f32 = 16.0;
pub const PROGENY_DISTANCE: f32 = 16.0;
pub const PROGENY_PLAY: f32 = 12.0;

pub const MINI_CARD_HEIGHT: f32 = 20.0;
pub const SINGLETON_BONDLESS: f32 = 0.0;

pub const MAX_SLIDE_DEPTH: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    pub vertical_calc: f32,
    pub little_calc: f32,
}

impl Default for Spacing {
    fn default() -> Spacing {
        Spacing {
            vertical_calc: VERTICAL_SPACE,
            little_calc: LITTLE_GROUP_DISTANCE,
        }
    }
}

impl Spacing {
    pub fn numbered(with_numbers: bool) -> Spacing {
        if with_numbers {
            Spacing::default()
        } else {
            Spacing {
                vertical_calc: VERTICAL_SPACE / 2.0,
                little_calc: LITTLE_GROUP_DISTANCE / 2.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_numbered() {
        let a = Spacing::default();
        let b = Spacing::numbered(true);
        assert_eq!(a.vertical_calc, b.vertical_calc);
        assert_eq!(a.little_calc, b.little_calc);
    }

    #[test]
    fn unnumbered_halves_gaps() {
        let s = Spacing::numbered(false);
        assert_eq!(s.vertical_calc, VERTICAL_SPACE / 2.0);
        assert_eq!(s.little_calc, LITTLE_GROUP_DISTANCE / 2.0);
    }

    #[test]
    fn constants_sane() {
        const { assert!(VERTICAL_SPACE > 0.0) }
        const { assert!(HORIZONTAL_SPACE > 0.0) }
        const { assert!(UNION_DISTANCE > HORIZONTAL_SPACE) }
        const { assert!(MARRIAGE_WIDTH > MARRIAGE_INNER_WIDTH) }
        const { assert!(BOND_WIDTH > MINI_BOND_WIDTH) }
        const { assert!(MINI_CARD_HEIGHT > 0.0) }
    }

    #[test]
    fn slide_depth_cap_is_bounded() {
        const { assert!(MAX_SLIDE_DEPTH > 0) }
        const { assert!(MAX_SLIDE_DEPTH < u32::MAX) }
    }
}
