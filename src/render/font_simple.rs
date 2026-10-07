use crate::render::font_file::Font;
use crate::render::font_read::{i16be, u16be};

const MAX_DEPTH: u8 = 4;

impl Font {
    fn byte_at(&self, p: usize, e: usize) -> Option<u8> {
        if p >= e {
            return Option::None;
        }
        self.data.get(p).copied()
    }

    fn word_at(&self, p: usize, e: usize) -> Option<u16> {
        if p.checked_add(2)? > e {
            return Option::None;
        }
        u16be(&self.data, p)
    }

    fn short_at(&self, p: usize, e: usize) -> Option<i16> {
        if p.checked_add(2)? > e {
            return Option::None;
        }
        i16be(&self.data, p)
    }

    pub(crate) fn simple(
        &self,
        o: usize,
        e: usize,
        n: usize,
    ) -> Option<Vec<Vec<(f32, f32, bool)>>> {
        let mut p = o.checked_add(10)?;
        if p.checked_add(n.checked_mul(2)?)? > e {
            return Option::None;
        }
        let mut ends: Vec<usize> = Vec::with_capacity(n);
        for _ in 0..n {
            ends.push(usize::from(self.word_at(p, e)?));
            p += 2;
        }
        let ins = usize::from(self.word_at(p, e)?);
        p = p.checked_add(2)?.checked_add(ins)?;
        let total = ends.last().copied().unwrap_or(0) + 1;
        let flags = self.read_flags(p, e, total)?;
        p = flags.next;
        let mut xs: Vec<f32> = Vec::with_capacity(total);
        let mut x = 0.0f32;
        for f in &flags.bytes {
            if f & 0x02 != 0 {
                let v = f32::from(self.byte_at(p, e)?);
                p += 1;
                x += if f & 0x10 != 0 { v } else { -v };
            } else if f & 0x10 == 0 {
                x += f32::from(self.short_at(p, e)?);
                p += 2;
            }
            xs.push(x);
        }
        let mut ys: Vec<f32> = Vec::with_capacity(total);
        let mut y = 0.0f32;
        for f in &flags.bytes {
            if f & 0x04 != 0 {
                let v = f32::from(self.byte_at(p, e)?);
                p += 1;
                y += if f & 0x20 != 0 { v } else { -v };
            } else if f & 0x20 == 0 {
                y += f32::from(self.short_at(p, e)?);
                p += 2;
            }
            ys.push(y);
        }
        let mut contours: Vec<Vec<(f32, f32, bool)>> = Vec::new();
        let mut s = 0usize;
        let last = total.saturating_sub(1);
        for end in ends {
            let mut c: Vec<(f32, f32, bool)> = Vec::new();
            for i in s..=end.min(last) {
                c.push((xs[i], ys[i], flags.bytes[i] & 0x01 != 0));
            }
            if !c.is_empty() {
                contours.push(c);
            }
            s = end.saturating_add(1);
        }
        Some(contours)
    }

    fn read_flags(&self, p: usize, e: usize, total: usize) -> Option<FlagRun> {
        let mut bytes: Vec<u8> = Vec::with_capacity(total);
        let mut p = p;
        while bytes.len() < total {
            let f = self.byte_at(p, e)?;
            p += 1;
            bytes.push(f);
            if f & 0x08 != 0 {
                let rep = usize::from(self.byte_at(p, e)?);
                p += 1;
                for _ in 0..rep {
                    bytes.push(f);
                }
            }
        }
        Some(FlagRun { bytes, next: p })
    }

    pub fn outline(&self, gid: u16, depth: u8) -> Vec<Vec<(f32, f32, bool)>> {
        self.outline_opt(gid, depth).unwrap_or_default()
    }

    fn outline_opt(&self, gid: u16, depth: u8) -> Option<Vec<Vec<(f32, f32, bool)>>> {
        if depth > MAX_DEPTH {
            return Option::None;
        }
        let (o, e, n) = self.glyph_header(gid)?;
        if n >= 0 {
            return self.simple(o, e, n as usize);
        }
        let mut out: Vec<Vec<(f32, f32, bool)>> = Vec::new();
        let mut p = o.checked_add(10)?;
        loop {
            if p.checked_add(4)? > e {
                break;
            }
            let flags = self.word_at(p, e)?;
            let comp = self.word_at(p.checked_add(2)?, e)?;
            p += 4;
            let mut dx = 0.0f32;
            let mut dy = 0.0f32;
            if flags & 0x0001 != 0 {
                if flags & 0x0002 != 0 {
                    dx = f32::from(self.short_at(p, e)?);
                    dy = f32::from(self.short_at(p.checked_add(2)?, e)?);
                    p += 4;
                } else {
                    p += 2;
                }
            } else if flags & 0x0002 != 0 {
                dx = f32::from(self.byte_at(p, e)?);
                dy = f32::from(self.byte_at(p.checked_add(1)?, e)?);
                p += 2;
            }
            let (a, b, c, d) = self.component_matrix(flags, &mut p, e)?;
            if flags & 0x0100 != 0 {
                let n = usize::from(self.word_at(p, e)?);
                p = p.checked_add(2)?.checked_add(n)?;
            }
            for contour in self.outline_opt(comp, depth.saturating_add(1))? {
                out.push(
                    contour
                        .into_iter()
                        .map(|(x, y, on)| (a * x + c * y + dx, b * x + d * y + dy, on))
                        .collect(),
                );
            }
            if flags & 0x0020 == 0 {
                break;
            }
        }
        Some(out)
    }

    fn component_matrix(
        &self,
        flags: u16,
        p: &mut usize,
        e: usize,
    ) -> Option<(f32, f32, f32, f32)> {
        let mut q = *p;
        let m = if flags & 0x0008 != 0 {
            let s = f32::from(self.short_at(q, e)?) / 16384.0;
            q += 2;
            (s, 0.0, 0.0, s)
        } else if flags & 0x0040 != 0 {
            let x = f32::from(self.short_at(q, e)?) / 16384.0;
            let y = f32::from(self.short_at(q.checked_add(2)?, e)?) / 16384.0;
            q += 4;
            (x, 0.0, 0.0, y)
        } else if flags & 0x0080 != 0 {
            let a = f32::from(self.short_at(q, e)?) / 16384.0;
            let b = f32::from(self.short_at(q.checked_add(2)?, e)?) / 16384.0;
            let c = f32::from(self.short_at(q.checked_add(4)?, e)?) / 16384.0;
            let d = f32::from(self.short_at(q.checked_add(6)?, e)?) / 16384.0;
            q += 8;
            (a, b, c, d)
        } else {
            (1.0, 0.0, 0.0, 1.0)
        };
        *p = q;
        Some(m)
    }
}

struct FlagRun {
    bytes: Vec<u8>,
    next: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real() -> Option<Font> {
        super::super::font_draw::Fonts::load().regular
    }

    #[test]
    fn unknown_gid_yields_empty() {
        if let Some(f) = real() {
            assert!(f.outline(60000, 0).is_empty());
            assert!(f.outline(0, 99).is_empty());
        }
    }

    #[test]
    fn simple_glyph_has_contours() {
        if let Some(f) = real() {
            let gid = f.glyph_id('A');
            if gid != 0 {
                let c = f.outline(gid, 0);
                assert!(!c.is_empty());
                for contour in &c {
                    assert!(contour.len() >= 3);
                }
            }
        }
    }

    #[test]
    fn all_glyphs_survive_sweep() {
        if let Some(f) = real() {
            for g in 0..=2048u16 {
                let _ = f.outline(g, 0);
            }
        }
    }

    #[test]
    fn missing_tables_yield_empty_outline() {
        assert!(Font::from_bytes(vec![0u8; 32]).is_none());
    }
}
