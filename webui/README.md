# pixelizer · webui

A browser front-end for the `pixelizer-core` pixel-art image-processing pipeline. Built in Rust and compiled to WebAssembly with [Leptos](https://leptos.dev/) (client-side rendering) and bundled by [Trunk](https://trunkrs.dev/). The UI lets you assemble an ordered pipeline of operations, load an image, run the pipeline, and view the result — all client-side, with no server and no JavaScript application code.

This crate is the `webui` member of the `pixelizer` Cargo workspace, alongside `core` (the processing library) and `cli`.

For planned features and work in progress, see [ROADMAP.md](ROADMAP.md).

## Status

The full data path works end to end: build a pipeline, upload an image, run it, see the output. The pipeline currently runs **synchronously on the main thread**, so the UI freezes for the duration of a run — noticeable on large images or expensive operation orders (e.g. posterize before downsample). Removing that freeze (via a web worker) is the main near-term task; see the ROADMAP.

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
├── main.rs              App root: shared state, Viewport, image decode/encode helpers, mount, Palettes loader
├── pipeline_list.rs     PipelineList: ordered list of op cards, add-operation control, id allocation, move/remove/edit handlers
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

- `rows` — the ordered pipeline, held as a `signal(Vec<OpRow>)` split into read/write halves (`rows: ReadSignal`, `set_rows: WriteSignal`) and passed to `PipelineList`. Each `OpRow` is an `{ id, op: Operation }`; the `id` is a stable key for Leptos's keyed `<For/>` so reordering animates correctly.
- `source: RwSignal<Option<Image>>` — the decoded source image (`pixelizer_core::Image`, an `RgbaImage`). Written by the Viewport's file input, read by the run handler.
- `output_url: RwSignal<Option<String>>` — the processed result as a PNG `data:` URL. Written by the run handler, read by the Viewport's `<img>`.

`PipelineList` owns the per-row mechanics: `next_id` (a `StoredValue` counter) allocates stable ids for new rows; `move_op` reorders by swapping; `remove_op` retains-by-id; and `edit_op` applies an incoming mutation closure to the matching row.

Editing an operation's settings goes through a single `on_edit` callback typed as `EditPayload = (usize, Box<dyn Fn(&mut Operation)>)`: a config card sends up the row id plus a closure that mutates the matching `Operation` in place. This keeps each card decoupled from how `rows` is stored.

The palette list is loaded once at startup from `palettes.yaml` (compiled into the binary via `include_str!`). `Palettes::load` parses it into a `HashMap<String, Vec<String>>`, then collects into a `Vec<(String, Vec<String>)>` sorted by name — so palettes always present in alphabetical order. The result is provided through Leptos context as a `StoredValue<Palettes>`, which the Palette Map config reads back with `use_context`.

### The run path

When **Run pipeline** is clicked:

1. The source `RgbaImage` is read from the `source` signal (the button is disabled while `source` is `None`).
2. The `rows` are collected into a `Pipeline { operations }`.
3. `pixelizer_core::apply(&pipeline, image)` runs the operations in order, each consuming the previous image by value and returning a new one.
4. The result is PNG-encoded, base64'd into a `data:` URL, and stored in `output_url`. Errors are logged rather than surfaced in the UI.
5. The Viewport's `<img>` reactively displays it.

Image decoding (`load_from_memory` → `to_rgba8`) and encoding (PNG → base64 data URL) are kept as plain functions free of any Leptos or DOM types. This isolation is deliberate: it's what will let the heavy work move into a web worker later with minimal disruption to the UI code.

## Dependencies of note

- `leptos` (csr) — reactive UI.
- `leptos-use` — `use_element_size` drives the op card's collapse animation by measuring content height.
- `gloo-file` — async reading of the uploaded file's bytes.
- `pixelizer-core` (workspace path dep) — the pipeline library; also re-exports `image`, so decode/encode stay on the same `image` version as core.
- `base64` — encoding the result for the `data:` URL.
- `yaml_serde` — parsing `palettes.yaml`.
- `wasm-bindgen-futures` — `spawn_local` for the async file read.
- `web-sys` — DOM types for the file input.
- `console_error_panic_hook` — readable panics in the browser console.

## Design notes

- **Rust/WASM as the primary medium**, with JavaScript treated as a thin interop edge only. No application logic lives in JS.
- **Owned-value pipeline**: each operation takes the image by value and returns a new one. This matches what the operations physically do (each allocates a new image, often of different dimensions) and means no in-place mutation or cloning inside `apply` — the image is *moved* through the chain.
- **State lifted to the root** so the run trigger, the pipeline, and the viewport coordinate only through parent-held signals.
