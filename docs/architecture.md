# Architecture

## Pipeline

A render runs five stages in order. The order matters because each stage feeds the next one.

1. Walk. `Graph::walk` starts from the fulcrum person and collects relatives into groups: parents above, siblings beside, spouses and children below, plus uncles, cousins, and second marriages up to the configured limits.
2. Measure. Every person node gets a pixel size from its name, titles, and dates. The engine never does this itself.
3. Init. `Animator::init_nodes` sizes family nodes from their partners, assigns rows per generation, links nodes into groups and unions, and creates connector lines.
4. Place. `Animator::place_nodes` positions everything: ancestors centered over descendants, spouses aligned, overlaps pushed apart over a bounded force loop, then the whole tree shifted so the minimum x and y are zero.
5. Draw. The render module walks the placed nodes and emits SVG: cards, borders, death ribbons, bonds, curves, dashed back lines, and duplicate-person connectors.

## Identity by index

The TypeScript original links objects with references. Cycles everywhere: nodes point at groups, groups point back at nodes, lines point at both. Rust cannot borrow that graph directly, so this port stores everything in vectors and links by `u32` index.

- `NodeId` is either `Person(u32)` or `Family(u32)`.
- `Animator` owns the vectors: persons, families, groups, unions, lines.
- `Graph` owns the `GedcomData` plus the `Animator` and the display settings.
- Methods that need two arenas at once take disjoint field borrows or copy small `Copy` values out first. There is no `Rc`, no `RefCell`, and no `unsafe` in the crate.

## Module map

| Directory | Contents |
| --------- | -------- |
| `config`  | `Card`, `Match`, `Branch`, `Side`, `Gender`, spacing constants |
| `model`   | `GedcomData`, person/family records, kinship counting, name/date text |
| `parser`  | line parser plus relation linking |
| `core`    | `NodeId`, shared node state, x/y setters, slide and overlap helpers |
| `nodes`   | person cards (birth/death counts for mini cards) and family nodes with bonds |
| `layout`  | groups (sibling rows), unions (ancestor columns), row operations |
| `lines`   | curve, vertical, horizontal, next, back, and duplicate connectors |
| `engine`  | `Graph` facade and node factories, tree walker, animator init and placement |
| `render`  | TrueType loading, text measurement, SVG output |

Files stay near 100 to 200 lines. When a module outgrows that, its `impl` blocks move to a sibling file by topic, which is why `engine` and `layout` have several files. Behavior that needs many pieces of `Animator` state lives in `impl Animator` blocks inside the relevant module file rather than in one giant file.
