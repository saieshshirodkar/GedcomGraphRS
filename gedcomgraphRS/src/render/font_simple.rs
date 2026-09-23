use crate::render::font_file::{Font, i16be, u16be};

impl Font {
    pub(crate) fn simple(
        &self,
        o: usize,
        e: usize,
        n: usize,
    ) -> Option<Vec<Vec<(f32, f32, bool)>>> {
        let mut p = o + 10;
        if p + n * 2 > e {
            return Option::None;
        }
        let mut ends: Vec<usize> = Vec::with_capacity(n);
        for _ in 0..n {
            ends.push(u16be(&self.data, p) as usize);
            p += 2;
        }
        if p + 2 > e {
            return Option::None;
        }
        let ins = u16be(&self.data, p) as usize;
        p += 2 + ins;
        let total = ends.last().copied().unwrap_or(0) + 1;
        let mut flags: Vec<u8> = Vec::with_capacity(total);
        while flags.len() < total {
            if p >= e {
                return Option::None;
            }
            let f = self.data[p];
            p += 1;
            flags.push(f);
            if f & 0x08 != 0 {
                if p >= e {
                    return Option::None;
                }
                let rep = self.data[p] as usize;
                p += 1;
                for _ in 0..rep {
                    flags.push(f);
                }
            }
        }
        let mut xs: Vec<f32> = Vec::with_capacity(total);
        let mut x = 0.0f32;
        for f in &flags {
            if f & 0x02 != 0 {
                if p >= e {
                    return Option::None;
                }
                let v = self.data[p] as f32;
                p += 1;
                x += if f & 0x10 != 0 { v } else { -v };
            } else if f & 0x10 == 0 {
                if p + 2 > e {
                    return Option::None;
                }
                x += i16be(&self.data, p) as f32;
                p += 2;
            }
            xs.push(x);
        }
        let mut ys: Vec<f32> = Vec::with_capacity(total);
        let mut y = 0.0f32;
        for f in &flags {
            if f & 0x04 != 0 {
                if p >= e {
                    return Option::None;
                }
                let v = self.data[p] as f32;
                p += 1;
                y += if f & 0x20 != 0 { v } else { -v };
            } else if f & 0x20 == 0 {
                if p + 2 > e {
                    return Option::None;
                }
                y += i16be(&self.data, p) as f32;
                p += 2;
            }
            ys.push(y);
        }
        let mut contours: Vec<Vec<(f32, f32, bool)>> = Vec::new();
        let mut s = 0usize;
        for end in ends {
            let mut c: Vec<(f32, f32, bool)> = Vec::new();
            for i in s..=end.min(total.saturating_sub(1)) {
                c.push((xs[i], ys[i], flags[i] & 0x01 != 0));
            }
            if !c.is_empty() {
                contours.push(c);
            }
            s = end + 1;
        }
        Some(contours)
    }

    pub fn outline(&self, gid: u16, depth: u8) -> Vec<Vec<(f32, f32, bool)>> {
        let mut out: Vec<Vec<(f32, f32, bool)>> = Vec::new();
        if depth > 4 {
            return out;
        }
        let Some((o, e)) = self.glyph_off(gid) else {
            return out;
        };
        if o + 10 > self.data.len() || e > self.data.len() {
            return out;
        }
        let n = i16be(&self.data, o);
        if n >= 0 {
            if let Some(c) = self.simple(o, e, n as usize) {
                out.extend(c);
            }
            return out;
        }
        let mut p = o + 10;
        loop {
            if p + 4 > e {
                break;
            }
            let flags = u16be(&self.data, p);
            let comp = u16be(&self.data, p + 2);
            p += 4;
            let mut dx = 0.0f32;
            let mut dy = 0.0f32;
            if flags & 0x0001 != 0 {
                if flags & 0x0002 != 0 {
                    dx = i16be(&self.data, p) as f32;
                    dy = i16be(&self.data, p + 2) as f32;
                    p += 4;
                } else {
                    p += 2;
                }
            } else if flags & 0x0002 != 0 {
                dx = self.data[p] as f32;
                dy = self.data[p + 1] as f32;
                p += 2;
            }
            let (a, b, c, d) = if flags & 0x0008 != 0 {
                let s = i16be(&self.data, p) as f32 / 16384.0;
                p += 2;
                (s, 0.0, 0.0, s)
            } else if flags & 0x0040 != 0 {
                let x = i16be(&self.data, p) as f32 / 16384.0;
                let y = i16be(&self.data, p + 2) as f32 / 16384.0;
                p += 4;
                (x, 0.0, 0.0, y)
            } else if flags & 0x0080 != 0 {
                let m = [
                    i16be(&self.data, p) as f32 / 16384.0,
                    i16be(&self.data, p + 2) as f32 / 16384.0,
                    i16be(&self.data, p + 4) as f32 / 16384.0,
                    i16be(&self.data, p + 6) as f32 / 16384.0,
                ];
                p += 8;
                (m[0], m[1], m[2], m[3])
            } else {
                (1.0, 0.0, 0.0, 1.0)
            };
            if flags & 0x0100 != 0 {
                let n = u16be(&self.data, p) as usize;
                p += 2 + n;
            }
            for contour in self.outline(comp, depth + 1) {
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
        out
    }
}
