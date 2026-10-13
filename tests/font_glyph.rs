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

#[test]
fn ascii_maps_even_when_format12_present() {
    let Some(f) = dejavu() else { return };
    for c in "Agz 0193.,'-".chars() {
        assert_ne!(f.glyph_id(c), 0, "ascii {c:?} unmapped");
    }
}

#[test]
fn format12_subtable_is_live() {
    let Some(f) = dejavu() else { return };
    assert_ne!(f.glyph_id('\u{1F600}'), 0, "format 12 cmap not consulted");
}
