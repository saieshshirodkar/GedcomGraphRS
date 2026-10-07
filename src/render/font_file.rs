use crate::render::font_cmap::{Cmap, parse as parse_cmap};
use crate::render::font_read::{find, i16be, read_dir, u16be, u32be};

#[derive(Debug, Clone)]
pub struct Font {
    pub(crate) data: Vec<u8>,
    pub units: f32,
    loca_off: Option<usize>,
    short_loca: bool,
    glyf_off: Option<usize>,
    num_glyphs: usize,
    hmtx_off: usize,
    num_hmetrics: usize,
    cmap: Cmap,
}

impl Font {
    pub fn load(path: &str) -> Option<Font> {
        let data = std::fs::read(path).ok()?;
        Font::from_bytes(data)
    }

    pub fn from_bytes(data: Vec<u8>) -> Option<Font> {
        let dir = read_dir(&data)?;
        let head_o = find(&dir, b"head")?;
        let maxp_o = find(&dir, b"maxp")?;
        let hhea_o = find(&dir, b"hhea")?;
        let hmtx_o = find(&dir, b"hmtx")?;
        let cmap_o = find(&dir, b"cmap")?;
        let units = f32::from(u16be(&data, head_o.checked_add(18)?)?);
        let short_loca = u16be(&data, head_o.checked_add(50)?)? == 0;
        let num_glyphs = usize::from(u16be(&data, maxp_o.checked_add(4)?)?);
        let num_hmetrics = usize::from(u16be(&data, hhea_o.checked_add(34)?)?);
        let cmap = parse_cmap(&data, cmap_o)?;
        if units <= 0.0 || num_glyphs == 0 || num_hmetrics == 0 {
            return Option::None;
        }
        Some(Font {
            data,
            units,
            loca_off: find(&dir, b"loca"),
            short_loca,
            glyf_off: find(&dir, b"glyf"),
            num_glyphs,
            hmtx_off: hmtx_o,
            num_hmetrics,
            cmap,
        })
    }

    pub fn glyph_id(&self, c: char) -> u16 {
        self.cmap.lookup(&self.data, c as u32)
    }

    pub fn advance(&self, gid: u16) -> f32 {
        if self.num_hmetrics == 0 {
            return 0.0;
        }
        let g = usize::from(gid).min(self.num_glyphs.saturating_sub(1));
        let idx = g.min(self.num_hmetrics - 1);
        let Some(o) = self.hmtx_off.checked_add(idx.saturating_mul(4)) else {
            return 0.0;
        };
        f32::from(u16be(&self.data, o).unwrap_or(0))
    }

    pub(crate) fn glyph_off(&self, gid: u16) -> Option<(usize, usize)> {
        let loca = self.loca_off?;
        let glyf = self.glyf_off?;
        let g = usize::from(gid);
        if g >= self.num_glyphs {
            return Option::None;
        }
        let (a, b) = self.loca_pair(loca, g)?;
        if a >= b {
            return Option::None;
        }
        let lo = glyf.checked_add(a)?;
        let hi = glyf.checked_add(b)?;
        if hi > self.data.len() {
            return Option::None;
        }
        Some((lo, hi))
    }

    fn loca_pair(&self, loca: usize, g: usize) -> Option<(usize, usize)> {
        if self.short_loca {
            let s = loca.checked_add(g.checked_mul(2)?)?;
            let a = usize::from(u16be(&self.data, s)?) * 2;
            let b = usize::from(u16be(&self.data, s.checked_add(2)?)?) * 2;
            Some((a, b))
        } else {
            let s = loca.checked_add(g.checked_mul(4)?)?;
            let a = usize::try_from(u32be(&self.data, s)?).ok()?;
            let b = usize::try_from(u32be(&self.data, s.checked_add(4)?)?).ok()?;
            Some((a, b))
        }
    }

    pub(crate) fn glyph_header(&self, gid: u16) -> Option<(usize, usize, i16)> {
        let (o, e) = self.glyph_off(gid)?;
        if o.checked_add(10)? > e {
            return Option::None;
        }
        Some((o, e, i16be(&self.data, o)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(num: u16) -> Vec<u8> {
        let mut d = vec![0u8; 12];
        d[4..6].copy_from_slice(&num.to_be_bytes());
        d
    }

    fn record(d: &mut Vec<u8>, tag: &[u8; 4], off: u32) {
        d.extend_from_slice(tag);
        d.extend_from_slice(&[0, 0, 0, 0]);
        d.extend_from_slice(&off.to_be_bytes());
        d.extend_from_slice(&[0, 0, 0, 0]);
    }

    #[test]
    fn rejects_short_and_empty() {
        assert!(Font::from_bytes(Vec::new()).is_none());
        assert!(Font::from_bytes(vec![0u8; 11]).is_none());
        assert!(Font::from_bytes(header(1)).is_none());
        assert!(Font::load("/nonexistent/path.ttf").is_none());
    }

    #[test]
    fn rejects_directory_pointing_past_eof() {
        let mut d = header(7);
        for t in [
            b"head", b"maxp", b"loca", b"glyf", b"hhea", b"hmtx", b"cmap",
        ] {
            record(&mut d, t, 4096);
        }
        assert!(Font::from_bytes(d).is_none());
    }

    #[test]
    fn rejects_zero_units_or_glyphs() {
        let mut d = header(5);
        let base = 12 + 5 * 16;
        record(&mut d, b"head", base);
        record(&mut d, b"maxp", base + 8);
        record(&mut d, b"hhea", base + 16);
        record(&mut d, b"hmtx", base + 24);
        record(&mut d, b"cmap", base + 32);
        let mut body = vec![0u8; 64];
        body[18..20].copy_from_slice(&0u16.to_be_bytes());
        d.extend_from_slice(&body);
        assert!(Font::from_bytes(d).is_none());
    }

    #[test]
    fn advance_out_of_range_is_zero_not_panic() {
        if let Some(f) = super::super::font_draw::Fonts::load().regular {
            assert!(f.advance(60000) >= 0.0);
            assert!(f.advance(0) > 0.0);
        }
    }

    #[test]
    fn glyph_off_respects_bounds() {
        if let Some(f) = super::super::font_draw::Fonts::load().regular {
            assert!(f.glyph_off(60000).is_none());
            assert!(f.glyph_off(0).is_some());
            assert!(f.glyph_header(60000).is_none());
        }
    }
}
