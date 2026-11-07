use crate::render::font_draw::Fonts;

const MAX_EMBED: usize = 6 * 1024 * 1024;
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const FAMILY: &str = "gedcomgraph";

fn mime_of(data: &[u8]) -> Option<(&'static str, &'static str)> {
    match data.get(0..4)? {
        [0x00, 0x01, 0x00, 0x00] => Some(("font/ttf", "truetype")),
        b"true" => Some(("font/ttf", "truetype")),
        b"OTTO" => Some(("font/otf", "opentype")),
        _ => Option::None,
    }
}

pub(crate) fn face_name(fonts: &Fonts) -> Option<&'static str> {
    let f = fonts.regular.as_ref()?;
    let (_, _) = mime_of(&f.data)?;
    if f.data.len() > MAX_EMBED {
        return Option::None;
    }
    Some(FAMILY)
}

pub(crate) fn font_face_style(fonts: &Fonts) -> Option<String> {
    let f = fonts.regular.as_ref()?;
    let (mime, format) = mime_of(&f.data)?;
    if f.data.len() > MAX_EMBED {
        return Option::None;
    }
    let mut s = String::with_capacity(f.data.len() * 4 / 3 + 160);
    s.push_str("<style>@font-face{font-family:'");
    s.push_str(FAMILY);
    s.push_str("';font-style:normal;font-weight:normal;src:url(data:");
    s.push_str(mime);
    s.push_str(";base64,");
    push_base64(&mut s, &f.data);
    s.push_str(") format('");
    s.push_str(format);
    s.push_str("');}</style>\n");
    Some(s)
}

pub(crate) fn push_base64(out: &mut String, data: &[u8]) {
    let mut i = 0usize;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | data[i + 2] as u32;
        for shift in [18, 12, 6, 0] {
            out.push(char::from(B64[((n >> shift) & 63) as usize]));
        }
        i += 3;
    }
    match data.len() - i {
        1 => {
            let n = (data[i] as u32) << 16;
            out.push(char::from(B64[((n >> 18) & 63) as usize]));
            out.push(char::from(B64[((n >> 12) & 63) as usize]));
            out.push_str("==");
        }
        2 => {
            let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
            out.push(char::from(B64[((n >> 18) & 63) as usize]));
            out.push(char::from(B64[((n >> 12) & 63) as usize]));
            out.push(char::from(B64[((n >> 6) & 63) as usize]));
            out.push('=');
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_vectors() {
        for (raw, want) in [
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
        ] {
            let mut s = String::new();
            push_base64(&mut s, raw.as_bytes());
            assert_eq!(s, want);
        }
        let mut s = String::new();
        push_base64(&mut s, b"");
        assert_eq!(s, "");
    }

    #[test]
    fn mime_detection() {
        assert_eq!(
            mime_of(&[0x00, 0x01, 0x00, 0x00, 1]).map(|m| m.0),
            Some("font/ttf")
        );
        assert_eq!(mime_of(b"OTTO\0\0\0\0").map(|m| m.1), Some("opentype"));
        assert_eq!(mime_of(b"ttcf\0\0\0\0"), Option::None);
        assert_eq!(mime_of(&[0x80, 0x01]), Option::None);
        assert_eq!(mime_of(&[]), Option::None);
    }
}
