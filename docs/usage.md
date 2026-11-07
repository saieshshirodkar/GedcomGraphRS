# Usage

## CLI

```sh
cargo run -- <input.ged> [output.svg] [scale] [personId]
```

`input.ged` is the only required argument. The rest work like this:

| Argument   | Default      | Meaning                                                |
| ---------- | ------------ | ------------------------------------------------------ |
| output.svg | none         | Write an SVG render instead of printing a text summary |
| scale      | 4            | Pixel scale of the render. Must parse as a number      |
| personId   | auto-picked  | GEDCOM id to start from, without `@` signs, like `I3`  |

Without an output file you get a text summary: canvas size, node counts, and one line per node with its generation and x position.

```sh
cargo run -- example.ged output.svg 4 I3
cargo run -- demo.ged demo.svg 4 I14
```

If the person id is missing or unknown, the program prints a warning to stderr and falls back to the first person who has both parents and a spouse. If nobody qualifies, it uses the first person in the file. An empty file exits with status 1.

## Render settings

The SVG path always uses the same configuration, which mirrors the original renderer:

- 10 ancestor generations, 10 great-uncle generations, 10 descendant generations
- 10 sibling/nephew and 10 uncle/cousin generations
- spouses, numbers, and duplicate lines on
- left-to-right layout
- bitmap grouping size 1000

You can change all of these through the `Graph` API if you use the crate as a library. The builder methods are `max_ancestors`, `max_great_uncles`, `max_descendants`, `max_siblings_nephews`, `max_uncles_cousins`, `display_spouses`, `display_numbers`, `display_duplicate_lines`, and `set_layout_direction`. Each returns `&mut Graph` so calls chain.

## Card sizes

The engine does not measure text on its own. Before layout, the render path measures every person node with the loaded fonts and calls `set_person_size` (or `set_all_person_sizes` for a uniform size). If you drive `Graph` yourself, set sizes before `init_nodes` or every card lays out at zero width.

`init_nodes` and `place_nodes` are both idempotent, so calling either twice leaves the layout and the rendered SVG unchanged. `start_from` resets the animator first, so walking a second person from the same `Graph` produces a clean tree.

## Fonts

Text measurement needs TrueType fonts at runtime. The loader walks the platform font directories searching for known family stems, preferring DejaVu Sans, then Liberation Sans, Noto Sans, FreeSans, Roboto, and the common system faces. It accepts `.ttf`, `.otf`, and `.ttc`, and reads both cmap format 4 and format 12 so wide characters resolve. Bold and italic fall back to the regular face when no separate face is found. The loaded regular face is embedded into the SVG as `@font-face` so rendered text matches measured widths without a system font.

Directories searched:

| Platform | Paths |
| --------- | ----- |
| Linux     | `/usr/share/fonts`, `/usr/local/share/fonts`, plus the `TTF`, `liberation`, `noto`, and `dejavu` subdirectories |
| macOS     | `/System/Library/Fonts`, `/System/Library/Fonts/Supplemental`, `/Library/Fonts` |
| Windows   | `C:\Windows\Fonts`, `C:\Windows\Fonts\truetype` |

DejaVu Sans is the closest match to what browsers and canvas use for `sans-serif`, which is why it is first. If no regular face loads, `Fonts::is_usable()` returns false and the CLI exits with status 1. Malformed fonts are rejected rather than fatal: every read is bounds-checked, so a truncated or corrupt file yields `None` instead of a panic. The `★`, `≈`, and `✛` date symbols come from whichever face has them, falling back across the loaded fonts per glyph.
