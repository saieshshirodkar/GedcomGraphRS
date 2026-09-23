# gedcomgraph

A family tree layout engine in Rust, ported 1:1 from a TypeScript project. You give it a GEDCOM file and a starting person. It figures out which relatives to show, lays them out in generations, and writes an SVG that looks like the FamilyGem dark theme.

There are no dependencies. The standard library is all it uses.

## Run it

```sh
cargo run -- example.ged output.svg 4 I3
```

That reads `example.ged`, starts from person `I3`, and writes `output.svg` at scale 4. The arguments after the input file are the output path, the scale, and the starting person id. Scale defaults to 4. If you skip the starting person, the program picks the first person who has both parents and a spouse.

Leave off the output file and you get a text summary instead:

```sh
cargo run -- example.ged
```

## Layout

- `docs/usage.md` explains the CLI and every rendering option.
- `docs/architecture.md` maps the source tree and the layout pipeline.
- `docs/gedcom.md` lists which GEDCOM tags the parser reads.
- `docs/development.md` covers building, testing, linting, and coverage.

## Repo layout

```text
src/
  config/    card, match, branch, side, gender enums and spacing constants
  model/     GEDCOM data model and kinship helpers
  parser/    GEDCOM text parser
  core/      node ids, shared node state, placement primitives
  nodes/     person and family nodes
  layout/    groups, unions, rows
  lines/     connector line segments
  engine/    graph facade, tree walker, layout animator
  render/    TTF text, SVG output
tests/       integration suites plus shared builders
docs/        you are here
example.ged  56-person tree across 7 generations used by the examples
output.svg   rendered output for example.ged
```
