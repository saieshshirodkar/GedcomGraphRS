pub const PAD: f32 = 40.0;

pub const COLOR_BG: [u8; 4] = [0, 0, 0, 255];
pub const COLOR_BACK_ELEMENT: [u8; 4] = [0x22, 0x22, 0x22, 255];
pub const COLOR_MALE: [u8; 4] = [0x44, 0xAA, 0xFF, 255];
pub const COLOR_FEMALE: [u8; 4] = [0xFF, 0x66, 0xDD, 255];
pub const COLOR_UNDEFINED: [u8; 4] = [0x99, 0x99, 0x99, 255];
pub const COLOR_PARTNER: [u8; 4] = [0, 0, 0, 0x66];
pub const COLOR_TEXT: [u8; 4] = [255, 255, 255, 255];
pub const COLOR_TEXT_VEILED: [u8; 4] = [255, 255, 255, 0xAA];
pub const COLOR_LINES: [u8; 4] = [0xDD, 0xDD, 0xDD, 255];
pub const COLOR_HEARTH: [u8; 4] = [0xDD, 0xDD, 0xDD, 255];
pub const COLOR_BACK_LINES: [u8; 4] = [0x55, 0x55, 0x55, 255];
pub const COLOR_DUP_DEFAULT: [u8; 4] = [0x99, 0x99, 0x99, 255];
pub const COLOR_RIBBON_WHITE: [u8; 4] = [255, 255, 255, 255];
pub const COLOR_RIBBON_BLACK: [u8; 4] = [0, 0, 0, 255];

#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub s: f32,
    pub name_px: f32,
    pub title_px: f32,
    pub date_px: f32,
    pub mini_px: f32,
    pub pad_x: f32,
    pub pad_top: f32,
    pub pad_bot: f32,
    pub lh_name: f32,
    pub lh_title: f32,
    pub lh_date: f32,
    pub card_min_w: f32,
    pub corner: f32,
    pub bg_corner: f32,
    pub border: f32,
    pub ribbon: f32,
}

impl Scale {
    pub fn of(s: f32) -> Scale {
        Scale {
            s,
            name_px: (17.0 * s).round(),
            title_px: (15.0 * s).round(),
            date_px: (14.0 * s).round(),
            mini_px: (11.0 * s).round(),
            pad_x: (6.0 * s).round(),
            pad_top: (5.0 * s).round(),
            pad_bot: (6.0 * s).round(),
            lh_name: (21.0 * s).round(),
            lh_title: (19.0 * s).round(),
            lh_date: (18.0 * s).round(),
            card_min_w: (50.0 * s).round(),
            corner: (3.0 * s).round(),
            bg_corner: (4.0 * s).round(),
            border: (2.0 * s).round(),
            ribbon: (18.0 * s).round(),
        }
    }

    pub fn sf(&self, v: f32) -> f32 {
        v * self.s
    }

    pub fn si(&self, v: f32) -> i32 {
        (v * self.s).round() as i32
    }

    pub fn pad_px(&self) -> i32 {
        (PAD * self.s).round() as i32
    }

    pub fn mini_box(&self) -> (f32, f32) {
        ((14.0 * self.s).round(), (8.0 * self.s).round())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_four_matches_ts() {
        let sc = Scale::of(4.0);
        assert_eq!(sc.name_px, 68.0);
        assert_eq!(sc.title_px, 60.0);
        assert_eq!(sc.date_px, 56.0);
        assert_eq!(sc.mini_px, 44.0);
        assert_eq!(sc.pad_x, 24.0);
        assert_eq!(sc.pad_top, 20.0);
        assert_eq!(sc.pad_bot, 24.0);
        assert_eq!(sc.lh_name, 84.0);
        assert_eq!(sc.lh_title, 76.0);
        assert_eq!(sc.lh_date, 72.0);
        assert_eq!(sc.card_min_w, 200.0);
        assert_eq!(sc.corner, 12.0);
        assert_eq!(sc.bg_corner, 16.0);
        assert_eq!(sc.border, 8.0);
        assert_eq!(sc.ribbon, 72.0);
        assert_eq!(sc.pad_px(), 160);
    }

    #[test]
    fn sf_si_math() {
        let sc = Scale::of(4.0);
        assert_eq!(sc.sf(2.5), 10.0);
        assert_eq!(sc.si(2.5), 10);
        assert_eq!(sc.si(2.51), 10);
        assert_eq!(sc.mini_box(), (56.0, 32.0));
    }

    #[test]
    fn palette_values() {
        assert_eq!(COLOR_BG, [0, 0, 0, 255]);
        assert_eq!(COLOR_MALE, [0x44, 0xAA, 0xFF, 255]);
        assert_eq!(COLOR_FEMALE, [0xFF, 0x66, 0xDD, 255]);
        assert_eq!(COLOR_PARTNER[3], 0x66);
        assert_eq!(COLOR_TEXT_VEILED[3], 0xAA);
        assert_eq!(PAD, 40.0);
    }
}
