# egui_areas

Blender-style area tiling for [egui](https://github.com/emilk/egui).

The window is a binary tree of **areas**. Each area shows one **editor** and has a
header. Drag an area corner to split it, join it into a neighbour, swap it with
another area, or pop it out; drag a divider to resize. Named layouts
(**workspaces**) can be switched, duplicated and saved, and tagged with a **mode** so
the right layout comes up for the kind of work you are doing.

The crate knows nothing about what your editors show. You provide that through one
trait, `AreaHost`.

## Features

| Feature | Default | Adds |
|---|---|---|
| `ui` | yes | The egui drawing layer, `egui_areas::ui`. |

Without `ui` the crate is the headless model: `AreaTree`, the operations that edit it
(`ops::split_area`, `join_areas`, `close_area`, `resize_split`, `swap_areas`, ...), `Workspace` and
`Workspaces`. It then depends only on `serde` and `slotmap`.

## Vocabulary

| Term | Meaning |
|---|---|
| `Area` | A leaf of the tree: one editor (`EditorKind`) and its view key (`ViewId`). |
| `AreaTree` | The tree of splits and areas. Splitting keeps the original area's `NodeId`. |
| `Workspace` | A named `AreaTree`, optionally tagged with a `mode`. |
| `Workspaces` | The user's set of layouts; never empty, names unique, serialises to TOML. |
| `AreaHost` | What you implement: editor names, header extras, drawing an editor. |
| `UnitRect` | A rectangle in unit space; the egui layer scales it to pixels. |

`EditorKind` is an open string, so your application owns the list of editors.
`ViewId` keys per-area view state (camera, tool, ...) that you keep outside the
crate; it follows an editor through swaps and is never reused.

## Sketch

```rust
use egui_areas::{AreaTree, Axis, EditorKind, Workspace, ops::split_area};

let mut tree = AreaTree::single(EditorKind::new("viewport"));
let viewport = tree.root();
let properties = split_area(&mut tree, viewport, Axis::Horizontal, 0.7, EditorKind::new("properties")).unwrap();
assert_eq!(tree.areas().len(), 2);

let mut workspace = Workspace::new("Modelling", tree);
workspace.mode = Some("modelling".into());
```

With the `ui` feature, draw it each frame. `host` implements `AreaHost`, and the
returned `TilingHint` says what the layout is offering right now (for a status bar):

```rust,ignore
let hint = egui_areas::ui::show_workspace(ui, &mut workspace, &mut active_area, &mut host);
```

## Status

Early: the API is not stable yet. The crate was extracted from a CAD application and
tracks egui's release cadence (currently egui 0.36).

## Licence

MIT.
