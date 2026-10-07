use std::fmt::Write as _;

pub(crate) fn push_u64(s: &mut String, mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    s.reserve(20 - i);
    for b in &buf[i..] {
        s.push(char::from(*b));
    }
}

#[inline]
fn rhe(x: f64) -> f64 {
    let f = x.floor();
    let d = x - f;
    let up = if d > 0.5 {
        true
    } else if d < 0.5 {
        false
    } else {
        (f as i64) % 2 != 0
    };
    if up { f + 1.0 } else { f }
}

fn scaled(v: f32, mul: f64) -> Option<i64> {
    if !v.is_finite() || v.abs() > 1.0e6 {
        return Option::None;
    }
    Some(rhe(f64::from(v) * mul) as i64)
}

pub(crate) fn push_fx1(s: &mut String, v: f32) {
    let Some(i) = scaled(v, 10.0) else {
        let _ = write!(s, "{v:.1}");
        return;
    };
    if i < 0 || (i == 0 && v.is_sign_negative()) {
        s.push('-');
    }
    let m = i.unsigned_abs();
    push_u64(s, m / 10);
    s.push('.');
    s.push(char::from(b'0' + (m % 10) as u8));
}

pub(crate) fn push_fx0(s: &mut String, v: f32) {
    let Some(i) = scaled(v, 1.0) else {
        let _ = write!(s, "{v:.0}");
        return;
    };
    if i < 0 || (i == 0 && v.is_sign_negative()) {
        s.push('-');
    }
    push_u64(s, i.unsigned_abs());
}

pub(crate) fn push_hex(s: &mut String, c: [u8; 4]) {
    s.reserve(7);
    s.push('#');
    for v in c.iter().take(3) {
        let _ = write!(s, "{:02X}", v);
    }
}

pub(crate) fn push_alpha(s: &mut String, c: [u8; 4]) {
    if c[3] != 255 {
        s.reserve(24);
        s.push_str(" fill-opacity=\"");
        let _ = write!(s, "{:.3}", c[3] as f32 / 255.0);
        s.push('"');
    }
}

pub(crate) fn push_esc(o: &mut String, s: &str) {
    o.reserve(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            _ => o.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_format() {
        let mut s = String::new();
        push_hex(&mut s, [0x44, 0xAA, 0xFF, 255]);
        assert_eq!(s, "#44AAFF");
        s.clear();
        push_hex(&mut s, [0, 0, 0, 0]);
        assert_eq!(s, "#000000");
    }

    #[test]
    fn alpha_opacity() {
        let mut s = String::new();
        push_alpha(&mut s, [255, 255, 255, 255]);
        assert_eq!(s, "");
        s.clear();
        push_alpha(&mut s, [0, 0, 0, 0x66]);
        assert_eq!(s, " fill-opacity=\"0.400\"");
    }

    #[test]
    fn esc_specials() {
        let mut s = String::new();
        push_esc(&mut s, "a&b<c>d");
        assert_eq!(s, "a&amp;b&lt;c&gt;d");
        s.clear();
        push_esc(&mut s, "plain");
        assert_eq!(s, "plain");
        s.clear();
        push_esc(&mut s, "");
        assert_eq!(s, "");
    }

    #[test]
    fn esc_does_not_double_escape() {
        let mut s = String::new();
        push_esc(&mut s, "a&amp;b");
        assert_eq!(s, "a&amp;amp;b");
    }

    #[test]
    fn fx1_matches_std_format() {
        let mut s: u32 = 22222;
        let mut checked = 0u32;
        for _ in 0..300_000 {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let v = f32::from_bits(s);
            if !v.is_finite() || v.abs() > 1.0e6 {
                continue;
            }
            checked += 1;
            let mut got = String::new();
            push_fx1(&mut got, v);
            assert_eq!(got, format!("{v:.1}"), "fx1 mismatch on {v:?}");
            got.clear();
            push_fx0(&mut got, v);
            assert_eq!(got, format!("{v:.0}"), "fx0 mismatch on {v:?}");
        }
        assert!(checked > 100_000, "sample too small: {checked}");
    }

    #[test]
    fn fx1_matches_std_on_dense_grid() {
        for i in -200_000i64..200_000 {
            let v = (i as f64 * 0.0007) as f32;
            let mut got = String::new();
            push_fx1(&mut got, v);
            assert_eq!(got, format!("{v:.1}"), "fx1 grid mismatch on {v:?}");
            got.clear();
            push_fx0(&mut got, v);
            assert_eq!(got, format!("{v:.0}"), "fx0 grid mismatch on {v:?}");
        }
    }

    #[test]
    fn fx_handles_edges() {
        for v in [
            0.0f32,
            -0.0,
            0.05,
            -0.05,
            0.25,
            0.35,
            1.05,
            2.5,
            -0.25,
            1e30,
            -1e30,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ] {
            let mut got = String::new();
            push_fx1(&mut got, v);
            assert_eq!(got, format!("{v:.1}"), "edge fx1 on {v:?}");
            got.clear();
            push_fx0(&mut got, v);
            assert_eq!(got, format!("{v:.0}"), "edge fx0 on {v:?}");
        }
    }

    #[test]
    fn push_u64_digits() {
        let mut s = String::new();
        push_u64(&mut s, 0);
        assert_eq!(s, "0");
        s.clear();
        push_u64(&mut s, 1);
        assert_eq!(s, "1");
        s.clear();
        push_u64(&mut s, 1_000_000);
        assert_eq!(s, "1000000");
        s.clear();
        push_u64(&mut s, u64::MAX);
        assert_eq!(s, "18446744073709551615");
    }
}
