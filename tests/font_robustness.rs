use gedcomgraph::render::font_draw::Fonts;
use gedcomgraph::render::font_file::Font;

fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}
fn be16(v: u16) -> [u8; 2] {
    v.to_be_bytes()
}

fn corrupt_dir(head_off: u32, body: usize) -> Vec<u8> {
    let tags: [&[u8; 4]; 7] = [
        b"head", b"maxp", b"loca", b"glyf", b"hhea", b"hmtx", b"cmap",
    ];
    let n = tags.len();
    let mut d = Vec::new();
    d.extend_from_slice(&be32(0x0001_0000));
    d.extend_from_slice(&be16(n as u16));
    for _ in 0..3 {
        d.extend_from_slice(&be16(0));
    }
    let base = 12 + n * 16;
    for (i, t) in tags.iter().enumerate() {
        let off = if *t == b"head" {
            head_off
        } else {
            (base + i) as u32
        };
        d.extend_from_slice(*t);
        d.extend_from_slice(&be32(0));
        d.extend_from_slice(&be32(off));
        d.extend_from_slice(&be32(4));
    }
    d.extend(std::iter::repeat_n(0u8, body));
    d
}

fn fuzz_bytes(seed: u32, len: usize) -> Vec<u8> {
    let mut s = seed.wrapping_mul(2654435761);
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        out.push((s >> 16) as u8);
    }
    out
}

#[test]
fn out_of_bounds_table_offsets_rejected_not_panicking() {
    for off in [4096u32, 65535, u32::MAX / 2, 152, 151] {
        let d = corrupt_dir(off, 64);
        assert!(Font::from_bytes(d).is_none(), "head_off={off} must reject");
    }
}

#[test]
fn truncation_at_every_length_never_panics() {
    let src = corrupt_dir(20, 4096);
    for n in 0..src.len().min(600) {
        let _ = Font::from_bytes(src[..n].to_vec());
    }
    let _ = Font::from_bytes(src);
}

#[test]
fn truncated_real_font_never_panics() {
    let path = format!(
        "{}/../../usr/share/fonts/TTF/DejaVuSans.ttf",
        env!("CARGO_MANIFEST_DIR")
    );
    let Ok(real) = std::fs::read(&path) else {
        return;
    };
    for n in (0..real.len().min(2000)).step_by(37) {
        let _ = Font::from_bytes(real[..n].to_vec());
    }
    for n in [real.len() / 4, real.len() / 2, real.len() - 1] {
        if let Some(f) = Font::from_bytes(real[..n].to_vec()) {
            for g in 0..200u16 {
                let _ = f.outline(g, 0);
                let _ = f.advance(g);
            }
        }
    }
}

#[test]
fn pseudo_random_garbage_never_panics() {
    for seed in 1..200u32 {
        let len = (seed as usize % 97) + 4;
        if let Some(f) = Font::from_bytes(fuzz_bytes(seed, len)) {
            for c in ['A', 'z', '0', 'é', '\u{1F600}'] {
                let g = f.glyph_id(c);
                let _ = f.advance(g);
                let _ = f.outline(g, 0);
                let _ = f.outline(g, 4);
                let _ = f.outline(g, 9);
            }
        }
    }
}

#[test]
fn single_byte_corruption_of_real_font_never_panics() {
    let path = format!(
        "{}/../../usr/share/fonts/TTF/DejaVuSans.ttf",
        env!("CARGO_MANIFEST_DIR")
    );
    let Ok(real) = std::fs::read(&path) else {
        return;
    };
    for i in (0..real.len().min(400)).step_by(11) {
        let mut d = real.clone();
        d[i] = d[i].wrapping_add(0x7F);
        if let Some(f) = Font::from_bytes(d) {
            for g in [0u16, 1, 36, 37, 100, 500, 2000, 60000] {
                let _ = f.outline(g, 0);
                let _ = f.advance(g);
            }
        }
    }
}

#[test]
fn system_font_sweep_never_panics() {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    collect(std::path::Path::new("/usr/share/fonts"), 0, &mut files);
    for f in files.iter().take(400) {
        let name = f.display().to_string();
        let Some(face) = Font::load(&name) else {
            continue;
        };
        assert!(face.advance(0) > 0.0);
        assert!(face.advance(60000) >= 0.0);
        for c in ['A', 'g', 'é', '中'] {
            let _ = face.advance(face.glyph_id(c));
        }
        for g in [0u16, 2, 36, 68, 1000, 60000] {
            let _ = face.outline(g, 0);
        }
    }
}

#[test]
fn wide_and_ttc_faces_measure_or_cleanly_reject() {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    collect(std::path::Path::new("/usr/share/fonts"), 0, &mut files);
    let mut otf = 0;
    let mut ttc = 0;
    for f in &files {
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("otf") {
            otf += 1;
        }
        if ext.eq_ignore_ascii_case("ttc") {
            ttc += 1;
        }
        let name = f.display().to_string();
        if let Some(face) = Font::load(&name) {
            assert!(face.units > 0.0);
            assert!(face.advance(face.glyph_id('A')) >= 0.0);
        }
    }
    assert!(otf + ttc > 0, "expected some otf/ttc files in the corpus");
}

#[test]
fn fonts_load_is_usable_when_a_face_exists() {
    let f = Fonts::load();
    if f.is_usable() {
        assert!(f.measure("Ag", 68.0, gedcomgraph::render::font_draw::Style::Regular) > 0.0);
        assert!(f.bold.is_some());
        assert!(f.fallback.is_some());
    } else {
        assert!(f.regular.is_none());
    }
    assert!(!Fonts::empty().is_usable());
}

fn collect(dir: &std::path::Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
    if depth > 5 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, depth + 1, out);
        } else if let Some(x) = p.extension().and_then(|e| e.to_str())
            && ["ttf", "otf", "ttc", "pfb"].contains(&x.to_ascii_lowercase().as_str())
        {
            out.push(p);
        }
    }
}
