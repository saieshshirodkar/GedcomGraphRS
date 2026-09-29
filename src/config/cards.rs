#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Card {
    Fulcrum = 0,
    #[default]
    Regular = 1,
    Ancestry = 2,
    Progeny = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Match {
    #[default]
    Main = 0,
    Near = 1,
    Middle = 2,
    Far = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Branch {
    #[default]
    None = 0,
    Pater = 1,
    Mater = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Side {
    #[default]
    None = 0,
    Left = 1,
    Right = 2,
}

impl Match {
    pub fn get(tot_families: usize, index: usize, side: Side, straight: bool) -> Match {
        if side == Side::Right {
            if index == 0 {
                return Match::Main;
            }
            if index == 1 && straight {
                return Match::Near;
            }
            if tot_families > 0 && index + 1 == tot_families {
                return Match::Far;
            }
        } else {
            if tot_families > 0 && index + 1 == tot_families {
                return Match::Main;
            }
            if tot_families >= 2 && index + 2 == tot_families && straight {
                return Match::Near;
            }
            if index == 0 {
                return Match::Far;
            }
        }
        Match::Middle
    }

    pub fn for_ancestors(tot_families: usize, index: usize, side: Side) -> Match {
        if (side == Side::Left && tot_families > 0 && index + 1 == tot_families)
            || (side == Side::Right && index == 0)
        {
            return Match::Near;
        }
        if (side == Side::Left && index == 0)
            || (side == Side::Right && tot_families > 0 && index + 1 == tot_families)
        {
            return Match::Far;
        }
        Match::Middle
    }

    pub fn is_multi(self) -> bool {
        self == Match::Far || self == Match::Middle || self == Match::Near
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_side_first_is_main() {
        assert_eq!(Match::get(3, 0, Side::Right, true), Match::Main);
    }

    #[test]
    fn right_side_second_straight_is_near() {
        assert_eq!(Match::get(3, 1, Side::Right, true), Match::Near);
    }

    #[test]
    fn right_side_second_bent_is_middle() {
        assert_eq!(Match::get(3, 1, Side::Right, false), Match::Middle);
    }

    #[test]
    fn right_side_last_is_far() {
        assert_eq!(Match::get(3, 2, Side::Right, true), Match::Far);
    }

    #[test]
    fn left_side_last_is_main() {
        assert_eq!(Match::get(3, 2, Side::Left, true), Match::Main);
    }

    #[test]
    fn left_side_middle_straight_is_near() {
        assert_eq!(Match::get(3, 1, Side::Left, true), Match::Near);
    }

    #[test]
    fn left_side_first_is_far() {
        assert_eq!(Match::get(3, 0, Side::Left, true), Match::Far);
    }

    #[test]
    fn ancestors_right_first_is_near() {
        assert_eq!(Match::for_ancestors(3, 0, Side::Right), Match::Near);
    }

    #[test]
    fn ancestors_right_last_is_far() {
        assert_eq!(Match::for_ancestors(3, 2, Side::Right), Match::Far);
    }

    #[test]
    fn ancestors_left_last_is_near() {
        assert_eq!(Match::for_ancestors(3, 2, Side::Left), Match::Near);
    }

    #[test]
    fn ancestors_left_first_is_far() {
        assert_eq!(Match::for_ancestors(3, 0, Side::Left), Match::Far);
    }

    #[test]
    fn ancestors_middle_is_middle() {
        assert_eq!(Match::for_ancestors(3, 1, Side::Left), Match::Middle);
        assert_eq!(Match::for_ancestors(3, 1, Side::Right), Match::Middle);
    }

    #[test]
    fn multi_match_detection() {
        assert!(!Match::Main.is_multi());
        assert!(Match::Near.is_multi());
        assert!(Match::Middle.is_multi());
        assert!(Match::Far.is_multi());
    }

    #[test]
    fn zero_family_edges() {
        assert_eq!(Match::get(0, 0, Side::Right, true), Match::Main);
        assert_eq!(Match::get(0, 0, Side::Left, true), Match::Far);
        assert_eq!(Match::for_ancestors(0, 0, Side::Right), Match::Near);
        assert_eq!(Match::for_ancestors(0, 0, Side::Left), Match::Far);
        assert_eq!(Match::get(1, 0, Side::Right, true), Match::Main);
        assert_eq!(Match::get(1, 0, Side::Left, true), Match::Main);
    }
}
