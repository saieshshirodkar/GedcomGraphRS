use crate::render::font_read::{at, i16be, u16be, u32be};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Seg4 {
    end: u16,
    start: u16,
    delta: i16,
    range_off: u16,
    range_pos: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Group12 {
    start: u32,
    end: u32,
    start_gid: u32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Cmap {
    pub segs: Vec<Seg4>,
    pub groups: Vec<Group12>,
}

const MAX_GROUPS: usize = 200_000;

impl Cmap {
    pub(crate) fn lookup(&self, d: &[u8], code: u32) -> u16 {
        if code > 0xFFFF {
            return self.lookup_group(code);
        }
        let v = code as u16;
        for s in &self.segs {
            if v > s.end {
                continue;
            }
            if v < s.start {
                continue;
            }
            if s.range_off == 0 {
                return v.wrapping_add(s.delta as u16);
            }
            let delta = usize::from(v - s.start);
            let Some(pos) = s
                .range_pos
                .checked_add(usize::from(s.range_off))
                .and_then(|p| p.checked_add(delta.checked_mul(2)?))
            else {
                return 0;
            };
            let Some(id) = u16be(d, pos) else { return 0 };
            if id == 0 {
                return 0;
            }
            return id.wrapping_add(s.delta as u16);
        }
        self.lookup_group(code)
    }

    fn lookup_group(&self, code: u32) -> u16 {
        for g in &self.groups {
            if code >= g.start && code <= g.end {
                let Some(id) = g.start_gid.checked_add(code - g.start) else {
                    return 0;
                };
                return u16::try_from(id).unwrap_or(0);
            }
        }
        0
    }

    fn absorb(&mut self, other: Cmap) {
        self.segs.extend(other.segs);
        self.groups.extend(other.groups);
    }
}

pub(crate) fn parse(d: &[u8], off: usize) -> Option<Cmap> {
    let n = usize::from(u16be(d, off.checked_add(2)?)?);
    let mut unicode: Option<Cmap> = Option::None;
    let mut other: Option<Cmap> = Option::None;
    let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for i in 0..n {
        let rec = off.checked_add(4)?.checked_add(i.checked_mul(8)?)?;
        let plat = u16be(d, rec)?;
        let enc = u16be(d, rec.checked_add(2)?)?;
        let sub = off.checked_add(usize::try_from(u32be(d, rec.checked_add(4)?)?).ok()?)?;
        if !seen.insert(sub) {
            continue;
        }
        let parsed = match u16be(d, sub)? {
            4 => parse4(d, sub),
            12 => parse12(d, sub),
            _ => Option::None,
        };
        let Some(parsed) = parsed else { continue };
        let pref = plat == 0 || (plat == 3 && (enc == 1 || enc == 10));
        if pref {
            match &mut unicode {
                Some(acc) => acc.absorb(parsed),
                slot => *slot = Some(parsed),
            }
        } else if other.is_none() {
            other = Some(parsed);
        }
    }
    unicode.or(other)
}

fn parse4(d: &[u8], sub: usize) -> Option<Cmap> {
    let seg_x2 = usize::from(u16be(d, sub.checked_add(6)?)?) / 2;
    if seg_x2 == 0 {
        return Option::None;
    }
    let ends = sub.checked_add(14)?;
    let starts = ends.checked_add(seg_x2.checked_mul(2)?.checked_add(2)?)?;
    let deltas = starts.checked_add(seg_x2.checked_mul(2)?)?;
    let ranges = deltas.checked_add(seg_x2.checked_mul(2)?)?;
    let mut segs: Vec<Seg4> = Vec::with_capacity(seg_x2.min(MAX_GROUPS));
    for s in 0..seg_x2 {
        let range_pos = ranges.checked_add(s.checked_mul(2)?)?;
        let Some(end) = u16be(d, ends.checked_add(s.checked_mul(2)?)?) else {
            break;
        };
        let Some(start) = u16be(d, starts.checked_add(s.checked_mul(2)?)?) else {
            break;
        };
        let Some(delta) = i16be(d, deltas.checked_add(s.checked_mul(2)?)?) else {
            break;
        };
        let Some(range_off) = u16be(d, range_pos) else {
            break;
        };
        segs.push(Seg4 {
            end,
            start,
            delta,
            range_off,
            range_pos,
        });
        if end == 0xFFFF {
            break;
        }
    }
    Some(Cmap {
        segs,
        groups: Vec::new(),
    })
}

fn parse12(d: &[u8], sub: usize) -> Option<Cmap> {
    let n = usize::try_from(u32be(d, sub.checked_add(12)?)?).ok()?;
    if n == 0 || n > MAX_GROUPS {
        return Option::None;
    }
    at(d, sub, 16)?;
    let mut groups: Vec<Group12> = Vec::with_capacity(n.min(4096));
    for g in 0..n {
        let o = sub.checked_add(16)?.checked_add(g.checked_mul(12)?)?;
        let (Some(start), Some(end), Some(start_gid)) = (
            u32be(d, o),
            u32be(d, o.checked_add(4)?),
            u32be(d, o.checked_add(8)?),
        ) else {
            break;
        };
        if start > end {
            continue;
        }
        groups.push(Group12 {
            start,
            end,
            start_gid,
        });
    }
    if groups.is_empty() {
        return Option::None;
    }
    Some(Cmap {
        segs: Vec::new(),
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_cmap_maps_to_zero() {
        let c = Cmap::default();
        assert_eq!(c.lookup(&[0u8; 4], 65), 0);
        assert_eq!(c.lookup(&[0u8; 4], 0x1F600), 0);
    }

    #[test]
    fn delta_segment_maps() {
        let c = Cmap {
            segs: vec![Seg4 {
                end: 0xFFFF,
                start: 0x0020,
                delta: -0x1E,
                range_off: 0,
                range_pos: 0,
            }],
            groups: Vec::new(),
        };
        assert_eq!(c.lookup(&[], 0x20), 2);
        assert_eq!(c.lookup(&[], 0x41), 0x23);
        assert_eq!(c.lookup(&[], 0x1F), 0);
    }

    #[test]
    fn group_maps_wide() {
        let c = Cmap {
            segs: Vec::new(),
            groups: vec![Group12 {
                start: 0x1F600,
                end: 0x1F64F,
                start_gid: 100,
            }],
        };
        assert_eq!(c.lookup(&[], 0x1F600), 100);
        assert_eq!(c.lookup(&[], 0x1F610), 116);
        assert_eq!(c.lookup(&[], 0x1F700), 0);
        assert_eq!(c.lookup(&[], 65), 0);
    }

    #[test]
    fn range_offset_segment_maps_via_data() {
        let data = [0u8, 0, 0, 0, 0, 90, 0, 1];
        let c = Cmap {
            segs: vec![Seg4 {
                end: 0x0041,
                start: 0x0041,
                delta: 0,
                range_off: 4,
                range_pos: 0,
            }],
            groups: Vec::new(),
        };
        assert_eq!(c.lookup(&data, 0x41), 90);
        assert_eq!(c.lookup(&[], 0x41), 0);
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse(&[], 0).is_none());
        assert!(parse(&[0u8; 8], 0).is_none());
        assert!(parse(&[0u8; 64], 60).is_none());
    }
}
