pub(crate) fn u16be(d: &[u8], o: usize) -> Option<u16> {
    let hi = *d.get(o)?;
    let lo = *d.get(o.checked_add(1)?)?;
    Some(((hi as u16) << 8) | lo as u16)
}

pub(crate) fn u32be(d: &[u8], o: usize) -> Option<u32> {
    let mut v: u32 = 0;
    for i in 0..4 {
        v = (v << 8) | *d.get(o.checked_add(i)?)? as u32;
    }
    Some(v)
}

pub(crate) fn i16be(d: &[u8], o: usize) -> Option<i16> {
    u16be(d, o).map(|v| v as i16)
}

pub(crate) fn at(d: &[u8], o: usize, count: usize) -> Option<&[u8]> {
    let end = o.checked_add(count)?;
    d.get(o..end)
}

const TTC_TAG: u32 = 0x7474_6366;
const MAX_TABLES: usize = 4096;
const TABLE_RECORD: usize = 16;

pub(crate) fn read_dir(d: &[u8]) -> Option<Vec<([u8; 4], usize)>> {
    let mut base = 0usize;
    if u32be(d, 0)? == TTC_TAG {
        base = usize::try_from(u32be(d, 12)?).ok()?;
    }
    let num = usize::from(u16be(d, base.checked_add(4)?)?);
    if num == 0 || num > MAX_TABLES {
        return Option::None;
    }
    let dir = base.checked_add(12)?;
    let end = dir.checked_add(num.checked_mul(TABLE_RECORD)?)?;
    if end > d.len() {
        return Option::None;
    }
    let mut out: Vec<([u8; 4], usize)> = Vec::with_capacity(num);
    for i in 0..num {
        let o = dir + i * TABLE_RECORD;
        let Some(tag) = at(d, o, 4) else { break };
        let mut name = [0u8; 4];
        name.copy_from_slice(tag);
        let Some(off) = u32be(d, o + 8).and_then(|v| usize::try_from(v).ok()) else {
            continue;
        };
        if off >= d.len() {
            continue;
        }
        out.push((name, off));
    }
    if out.is_empty() {
        return Option::None;
    }
    Some(out)
}

pub(crate) fn find(dir: &[([u8; 4], usize)], name: &[u8; 4]) -> Option<usize> {
    dir.iter().find(|t| &t.0 == name).map(|t| t.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_bounds_reads() {
        let d = [0x00, 0x01, 0x00, 0x02, 0xFF, 0xFF];
        assert_eq!(u16be(&d, 0), Some(1));
        assert_eq!(u16be(&d, 2), Some(2));
        assert_eq!(i16be(&d, 4), Some(-1));
        assert_eq!(u32be(&d, 0), Some(0x0001_0002));
        assert_eq!(u32be(&d, 2), Some(0x0002_FFFF));
    }

    #[test]
    fn out_of_bounds_is_none() {
        let d = [0x00, 0x01, 0x00];
        assert_eq!(u16be(&d, 2), None);
        assert_eq!(u16be(&d, 3), None);
        assert_eq!(u32be(&d, 0), None);
        assert_eq!(i16be(&d, usize::MAX), None);
        assert_eq!(at(&d, 2, 5), None);
        assert_eq!(read_dir(&d), None);
    }

    #[test]
    fn empty_and_short_reject() {
        assert_eq!(read_dir(&[]), None);
        assert_eq!(u16be(&[], 0), None);
        let mut tiny = [0u8; 12];
        tiny[4] = 0;
        tiny[5] = 1;
        assert_eq!(read_dir(&tiny), None);
    }

    #[test]
    fn dir_skips_out_of_range_offsets() {
        let mut d = vec![0u8; 12 + 2 * TABLE_RECORD];
        d[4] = 0;
        d[5] = 2;
        for (i, tag) in [b"head", b"cmap"].iter().enumerate() {
            let o = 12 + i * TABLE_RECORD;
            d[o..o + 4].copy_from_slice(*tag);
            let off = if i == 0 { 40u32 } else { 9999u32 };
            d[o + 8..o + 12].copy_from_slice(&off.to_be_bytes());
        }
        let dir = read_dir(&d).unwrap();
        assert_eq!(dir.len(), 1);
        assert_eq!(find(&dir, b"head"), Some(40));
        assert_eq!(find(&dir, b"cmap"), None);
    }

    #[test]
    fn truncated_dir_rejects() {
        let mut d = vec![0u8; 12];
        d[4] = 0;
        d[5] = 200;
        assert_eq!(read_dir(&d), None);
    }

    #[test]
    fn ttc_base_offset_used() {
        let mut d = vec![0u8; 96];
        d[0..4].copy_from_slice(&TTC_TAG.to_be_bytes());
        d[12..16].copy_from_slice(&32u32.to_be_bytes());
        d[36] = 0;
        d[37] = 1;
        d[44..48].copy_from_slice(b"head");
        d[52..56].copy_from_slice(&60u32.to_be_bytes());
        let dir = read_dir(&d).unwrap();
        assert_eq!(find(&dir, b"head"), Some(60));
    }
}
