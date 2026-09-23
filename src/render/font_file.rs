pub fn u16be(d: &[u8], o: usize) -> u16 {
    ((d[o] as u16) << 8) | d[o + 1] as u16
}

pub fn u32be(d: &[u8], o: usize) -> u32 {
    ((d[o] as u32) << 24) | ((d[o + 1] as u32) << 16) | ((d[o + 2] as u32) << 8) | d[o + 3] as u32
}

pub fn i16be(d: &[u8], o: usize) -> i16 {
    u16be(d, o) as i16
}

#[derive(Debug, Clone, Copy)]
struct CmapSeg {
    end: u16,
    start: u16,
    delta: i16,
    range_off: u16,
    range_pos: usize,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub(crate) data: Vec<u8>,
    pub units: f32,
    loca_off: usize,
    short_loca: bool,
    glyf_off: usize,
    num_glyphs: usize,
    hmtx_off: usize,
    num_hmetrics: usize,
    segs: Vec<CmapSeg>,
}

impl Font {
    pub fn load(path: &str) -> Option<Font> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 12 {
            return Option::None;
        }
        let num = u16be(&data, 4) as usize;
        let mut tables: Vec<([u8; 4], usize, usize)> = Vec::new();
        for i in 0..num {
            let o = 12 + i * 16;
            if o + 16 > data.len() {
                return Option::None;
            }
            let tag = [data[o], data[o + 1], data[o + 2], data[o + 3]];
            tables.push((
                tag,
                u32be(&data, o + 8) as usize,
                u32be(&data, o + 12) as usize,
            ));
        }
        let find = |name: &[u8; 4]| tables.iter().find(|t| &t.0 == name).map(|t| (t.1, t.2));
        let (head_o, _) = find(b"head")?;
        let (maxp_o, _) = find(b"maxp")?;
        let (loca_o, _) = find(b"loca")?;
        let (glyf_o, _) = find(b"glyf")?;
        let (hhea_o, _) = find(b"hhea")?;
        let (hmtx_o, _) = find(b"hmtx")?;
        let (cmap_o, cmap_len) = find(b"cmap")?;
        let units = u16be(&data, head_o + 18) as f32;
        let short_loca = u16be(&data, head_o + 50) == 0;
        let num_glyphs = u16be(&data, maxp_o + 4) as usize;
        let num_hmetrics = u16be(&data, hhea_o + 34) as usize;
        let segs = parse_cmap(&data, cmap_o, cmap_len)?;
        if units <= 0.0 || num_glyphs == 0 {
            return Option::None;
        }
        Some(Font {
            data,
            units,
            loca_off: loca_o,
            short_loca,
            glyf_off: glyf_o,
            num_glyphs,
            hmtx_off: hmtx_o,
            num_hmetrics,
            segs,
        })
    }

    pub fn glyph_id(&self, c: char) -> u16 {
        let code = c as u32;
        if code > 0xFFFF {
            return 0;
        }
        let v = code as u16;
        for s in &self.segs {
            if v > s.end {
                continue;
            }
            if v < s.start {
                break;
            }
            if s.range_off == 0 {
                return v.wrapping_add(s.delta as u16);
            }
            let pos = s.range_pos + s.range_off as usize + ((v - s.start) as usize) * 2;
            if pos + 2 > self.data.len() {
                return 0;
            }
            let id = u16be(&self.data, pos);
            if id == 0 {
                return 0;
            }
            return id.wrapping_add(s.delta as u16);
        }
        0
    }

    pub fn advance(&self, gid: u16) -> f32 {
        let g = (gid as usize).min(self.num_glyphs.saturating_sub(1));
        let idx = g.min(self.num_hmetrics.saturating_sub(1));
        u16be(&self.data, self.hmtx_off + idx * 4) as f32
    }

    pub(crate) fn glyph_off(&self, gid: u16) -> Option<(usize, usize)> {
        let g = gid as usize;
        if g >= self.num_glyphs {
            return Option::None;
        }
        let (a, b) = if self.short_loca {
            (
                u16be(&self.data, self.loca_off + g * 2) as usize * 2,
                u16be(&self.data, self.loca_off + g * 2 + 2) as usize * 2,
            )
        } else {
            (
                u32be(&self.data, self.loca_off + g * 4) as usize,
                u32be(&self.data, self.loca_off + g * 4 + 4) as usize,
            )
        };
        if a == b {
            return Option::None;
        }
        Some((self.glyf_off + a, self.glyf_off + b))
    }
}

fn parse_cmap(data: &[u8], off: usize, len: usize) -> Option<Vec<CmapSeg>> {
    if off + 4 > data.len() {
        return Option::None;
    }
    let n = u16be(data, off + 2) as usize;
    for i in 0..n {
        let r = off + 4 + i * 8;
        if r + 8 > off + len || r + 8 > data.len() {
            break;
        }
        let fmt = u16be(data, u32be(data, r + 4) as usize + off);
        let sub = u32be(data, r + 4) as usize + off;
        if fmt != 4 {
            continue;
        }
        let seg_x2 = u16be(data, sub + 6) as usize / 2;
        let mut segs: Vec<CmapSeg> = Vec::new();
        for s in 0..seg_x2 {
            let end = u16be(data, sub + 14 + s * 2);
            let start = u16be(data, sub + 16 + seg_x2 * 2 + s * 2);
            let delta = i16be(data, sub + 16 + seg_x2 * 4 + s * 2);
            let roff_pos = sub + 16 + seg_x2 * 6 + s * 2;
            let range_off = u16be(data, roff_pos);
            segs.push(CmapSeg {
                end,
                start,
                delta,
                range_off,
                range_pos: roff_pos,
            });
            if end == 0xFFFF {
                break;
            }
        }
        return Some(segs);
    }
    Option::None
}
