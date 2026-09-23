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

The SVG path always uses the same configuration, which mirrors the original TypeScript renderer:

- 10 ancestor generations, 10 great-uncle generations, 10 descendant generations
- 10 sibling/nephew and 10 uncle/cousin generations
- spouses, numbers, and duplicate lines on
- left-to-right layout
- bitmap grouping size 1000

You can change all of these through the `Graph` API if you use the crate as a library. The builder methods are `max_ancestors`, `max_great_uncles`, `max_descendants`, `max_siblings_nephews`, `max_uncles_cousins`, `display_spouses`, `display_numbers`, `display_duplicate_lines`, and `set_layout_direction`. Each returns `&mut Graph` so calls chain.

## Card sizes

The engine does not measure text on its own. Before layout, the render path measures every person node with the loaded fonts and calls `set_person_size` (or `set_all_person_sizes` for a uniform size). If you drive `Graph` yourself, set sizes before `init_nodes` or every card lays out at zero width.

## Fonts

Text measurement needs TrueType fonts at runtime. The loader looks in `/usr/share/fonts/TTF`, `/usr/share/fonts/liberation`, and `/usr/share/fonts/noto`, trying DejaVu Sans first, then Liberation Sans, then Noto Sans, with separate bold and italic faces. DejaVu Sans is the closest match to what browsers and canvas use for `sans-serif`, which is why it is first. If no regular face loads, the program exits with status 1. The `★`, `≈`, and `✛` date symbols come from whichever face has them, falling back across the loaded fonts per glyph.
