# pixelizer · webui

A browser front-end for the `pixelizer-core` pixel-art image-processing pipeline. Built in Rust and compiled to WebAssembly with [Leptos](https://leptos.dev/) (client-side rendering) and bundled by [Trunk](https://trunkrs.dev/). The UI lets you assemble an ordered pipeline of operations, load an image, run the pipeline, and view the result — all client-side, with no server and no JavaScript application code.

This crate is the `webui` member of the `pixelizer` Cargo workspace, alongside `core` (the processing library) and `cli`.

## Status

Phase 1 is complete: the full data path works end to end. You can build a pipeline, upload an image, run it, and see the output. The pipeline currently runs **synchronously on the main thread**, so the UI freezes for the duration of a run — noticeable on large images or expensive operation orders (e.g. posterize before downsample). Removing that freeze is the main near-term task (see [Web workers](#web-workers-planned)).

## Running locally

From the `webui/` directory:

```
trunk serve
```

Then open `http://localhost:8080`.

To view from another device on the same LAN (a phone, another laptop), bind to all interfaces and visit your machine's LAN IP:

```
trunk serve --address 0.0.0.0
```

Find your LAN IP with `ip addr show | grep "inet "` (look for a `192.168.x.x` or `10.x.x.x` address) and browse to `http://<that-ip>:8080`. Note the layout is currently desktop-oriented and not yet adapted for small screens.

## Building for release

Trunk follows the same dev/release split as `cargo build`. The default is the **dev** profile — unoptimized, which for WebAssembly means both slower execution and a noticeably larger `.wasm` download. For an image-processing app this difference is significant, so any meaningful performance testing (and anything shared over the LAN) should use a release build:

```
trunk build --release      # produce an optimized bundle in dist/
trunk serve --release      # serve an optimized build while developing
```

Without `--release` you are measuring an artificially slow binary; a release build of the pipeline can be several times faster.

There are two further size/speed levers beyond the flag, both WebAssembly-specific:

**Rust release profile** — tune in the workspace root `Cargo.toml`:

```toml
[profile.release]
opt-level = "s"     # "s"/"z" optimize for size; "3" (default) favors speed
lto = true          # link-time optimization — smaller and faster, near-free win
codegen-units = 1   # slightly better optimization, slower build
```

For this app the compute is the bottleneck rather than the download, so `opt-level = "3"` may be the better choice — worth measuring both against a representative image. `lto = true` is worth enabling regardless.

**`wasm-opt`** — a post-processing pass (from Binaryen) that optimizes the `.wasm` after `rustc` is done, often shrinking it further. Trunk can run it as part of the build; configure it in `Trunk.toml`. See the [Trunk documentation](https://trunkrs.dev/) for the current option names.

## How it works

### Module layout

```
src/
├── main.rs              App root: shared state, Viewport, image decode/encode helpers, mount
├── pipeline_list.rs     PipelineList: the ordered list of op cards + add-operation control
├── op_card.rs           OpCard: one card — header bar + collapsible animated settings area
└── op_card/
    ├── config.rs        op_config_view: dispatches an Operation to its config sub-view
    └── config/
        ├── blur.rs
        ├── downsample.rs
        ├── normalize.rs
        ├── number_slider.rs   Reusable labeled slider used across several cards
        ├── palette_map.rs
        ├── posterize.rs
        └── upscale.rs
```

### State and data flow

Shared state lives at the `App` root and flows down to children, so siblings never reach into one another:

- `rows: Signal<Vec<OpRow>>` — the ordered pipeline. Each `OpRow` is an `{ id, op: Operation }`; the `id` is a stable key for Leptos's keyed `<For/>` so reordering animates correctly. Owned by `App`, passed to `PipelineList` as read/write halves.
- `source: RwSignal<Option<Image>>` — the decoded source image (`pixelizer_core::Image`, an `RgbaImage`). Written by the Viewport's file input, read by the run handler.
- `output_url: RwSignal<Option<String>>` — the processed result as a PNG `data:` URL. Written by the run handler, read by the Viewport's `<img>`.

Editing an operation's settings goes through a single `on_edit` callback typed as `EditPayload = (usize, Box<dyn Fn(&mut Operation)>)`: a config card sends up the row id plus a closure that mutates the matching `Operation` in place. This keeps each card decoupled from how `rows` is stored.

The palette list is loaded once at startup from `palettes.yaml` (compiled into the binary via `include_str!`), parsed into a `Palettes` struct, and provided through Leptos context as a `StoredValue<Palettes>`. The Palette Map config reads it back with `use_context`.

### The run path

When **Run pipeline** is clicked:

1. The source `RgbaImage` is read from the `source` signal.
2. The `rows` are collected into a `Pipeline { operations }`.
3. `pixelizer_core::apply(&pipeline, image)` runs the operations in order, each consuming the previous image by value and returning a new one.
4. The result is PNG-encoded, base64'd into a `data:` URL, and stored in `output_url`.
5. The Viewport's `<img>` reactively displays it.

Image decoding (`load_from_memory` → `to_rgba8`) and encoding (PNG → base64 data URL) are kept as plain functions free of any Leptos or DOM types. This isolation is deliberate: it's what will let the heavy work move into a web worker later with minimal disruption to the UI code.

## Dependencies of note

- `leptos` (csr) — reactive UI.
- `leptos-use` — `use_element_size` drives the op card's collapse animation by measuring content height.
- `gloo-file` — async reading of the uploaded file's bytes.
- `image` (re-exported from `pixelizer-core`) — decode/encode, kept on the same version as core.
- `base64` — encoding the result for the `data:` URL.
- `yaml_serde` — parsing `palettes.yaml`.

## Planned features

The following are not yet implemented. Roughly ordered from most self-contained to most involved.

### Viewport improvements

- **Show the source image immediately on upload**, before any run. Currently nothing is displayed until the pipeline produces an `output_url`; the Viewport should fall back to rendering the `source` image when no output exists yet.
- **Fit the image within the visible area** so it never overflows the screen, regardless of its dimensions.

### Palette Map op card

This is the most feature-dense card and the main UI build-out:

- **Color swatches**: render the current palette as a row of colored squares instead of (or alongside) the dropdown.
- **Add-swatch affordance**: an empty `+` swatch the user clicks to add a new color.
- **Custom color picker**: clicking a swatch (including the `+`) opens a color picker. The native browser picker is inadequate, so this is a from-scratch component.
- **Dithering configuration**: surface the `DitherConfig` options (Floyd–Steinberg, Atkinson, JJN, Bayer4/8, with their `bleed`/`clamp`/`strength` parameters) on this same card.

### Droppable palette files

Allow dropping in a palette file so colors don't have to be chosen one at a time. Today the only palettes available are the pre-compiled options baked in from `palettes.yaml`; this would let users supply their own at runtime.

### Drag-and-drop reordering

Make op cards reorderable by dragging, behaving like dnd-kit's sortable (pointer-driven drag with a lifted card and the rest animating out of the way). The current up/down arrow buttons are the placeholder for this. The card header bar is already marked as the future drag handle.

### Per-op preview

Let the user click (or hover) an op card to see what that specific step does to the image — an intermediate preview of the pipeline up to and including that operation.

### Web workers (planned)

The pipeline currently runs on the main thread, freezing the UI for the length of a run. Moving it to a web worker keeps the UI responsive. A web worker is a separate thread with no shared memory, so this is a request/response restructuring rather than a drop-in swap. Rough shape:

1. **A worker entry point** — a separate compiled artifact the browser loads independently. It receives a message containing the source image bytes and the pipeline, runs the existing decode → apply → encode chain, and posts the resulting PNG data URL back. The already-isolated, DOM-free helper functions move here largely unchanged.
2. **The run handler becomes a send**, not a compute. Instead of calling `apply` inline, it serializes the inputs and posts them to the worker, then returns immediately — this is what removes the freeze.
3. **A result handler** receives the worker's reply and writes `output_url`. The write-to-signal migrates out of the click handler and into this message handler, because the result now arrives asynchronously rather than as a return value.

What crosses the boundary is serialized, so a live `RgbaImage` can't be sent; the plan is to send the original encoded file bytes plus the pipeline and let the worker own the whole decode/apply/encode chain, sending back a string. The [`gloo-worker`](https://docs.rs/gloo-worker/) crate provides a typed request/response abstraction over raw `postMessage` and is the likely path. This is the fiddliest part of the project — it requires a second build target and getting Trunk to emit and serve both artifacts.

## Design notes

- **Rust/WASM as the primary medium**, with JavaScript treated as a thin interop edge only. No application logic lives in JS.
- **Owned-value pipeline**: each operation takes the image by value and returns a new one. This matches what the operations physically do (each allocates a new image, often of different dimensions) and means no in-place mutation or cloning inside `apply` — the image is *moved* through the chain.
- **State lifted to the root** so the run trigger, the pipeline, and the viewport coordinate only through parent-held signals.
