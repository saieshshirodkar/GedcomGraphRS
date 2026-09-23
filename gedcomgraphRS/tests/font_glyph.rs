use gedcomgraph::render::font_file::Font;

fn dejavu() -> Option<Font> {
    Font::load("/usr/share/fonts/TTF/DejaVuSans.ttf")
}

#[test]
fn simple_outline_parses() {
    let Some(f) = dejavu() else { return };
    let o = f.outline(f.glyph_id('A'), 0);
    assert!(!o.is_empty());
    assert!(o.iter().map(|c| c.len()).sum::<usize>() > 4);
}

#[test]
fn composite_outline_parses() {
    let Some(f) = dejavu() else { return };
    let o = f.outline(f.glyph_id('é'), 0);
    assert!(!o.is_empty());
}

#[test]
fn empty_and_missing_safe() {
    let Some(f) = dejavu() else { return };
    assert!(f.outline(0xFFFF, 0).is_empty());
    assert!(f.outline(f.glyph_id('A'), 9).is_empty());
    assert!(f.glyph_id('\u{10FFFF}') == 0);
}
