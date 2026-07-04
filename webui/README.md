# pixelizer · webui

A browser front-end for the `pixelizer-core` pixel-art pipeline. Rust compiled to WebAssembly via [Leptos](https://leptos.dev/) (CSR), bundled by [Trunk](https://trunkrs.dev/). Assemble an ordered pipeline of operations, load an image, run it, view the result — entirely client-side, no server, no application JavaScript.

The `webui` member of the `pixelizer` workspace, alongside `core` (the pipeline library) and `cli`.

Planned work: [ROADMAP.md](ROADMAP.md).

## Status

Full path works: build a pipeline, upload an image, run, see output. The pipeline runs **synchronously on the main thread**, so the UI freezes for the length of a run — noticeable on large images or expensive orderings (posterize before downsample). Removing that freeze via a web worker is the headline remaining task (ROADMAP).

## Running locally

From `webui/`:

```
trunk serve                       # http://localhost:8080
trunk serve --address 0.0.0.0     # reachable from other LAN devices
```

For LAN access, find your IP with `ip addr show | grep "inet "` (a `192.168.x.x` / `10.x.x.x`) and browse to `http://<ip>:8080`. Layout is desktop-oriented; small screens are a known gap.

## Building for release

Trunk mirrors `cargo`'s dev/release split, and the gap matters more here than usual: a dev `.wasm` is both slower to run and larger to download. **Measure and share only release builds** — dev numbers are meaningless for an image pipeline.

```
trunk build --release
trunk serve --release
```

Two WASM-specific levers beyond the flag:

**Release profile** (workspace-root `Cargo.toml`):

```toml
[profile.release]
opt-level = "s"     # "s"/"z" = size; "3" (default) = speed
lto = true          # near-free size+speed win
codegen-units = 1   # marginally better codegen, slower build
```

Compute is the bottleneck here, not download, so `opt-level = "3"` is likely better — measure both against a representative image. `lto = true` regardless.

**`wasm-opt`** (Binaryen) — post-`rustc` pass, shrinks the `.wasm` further; Trunk runs it, configured in `Trunk.toml`. See Trunk docs for current option names.

## Architecture

The central design decision — the one to re-load into your head first — is that **the live pipeline is stored as data, not as core's typed `Operation` enum.** Everything below follows from that.

### Two representations, one boundary

`core::Operation` is the typed form the pipeline needs at `apply` time. But using it as the *live UI state* forces a translation layer on every edit: a generic slider doesn't know a `Blur` has a `sigma: f32`, so editing it meant serializing the op, poking one field, and deserializing back (an earlier design did exactly this, via `serde_json`; it's gone).

Instead the live state is a **value bag**: an op-instance is a `tag` plus a `BTreeMap<String, ParamValue>` keyed by the same param keys the schema declares. A widget reads and writes `values[key]` directly — no closures, no serde. The typed `Operation` is reconstructed **once, at Run**, and that boundary is the only place the schema-vs-bag contract is checked.

The two data sources:

- **Schema** — `core::op_schema`, a `'static` descriptor table (`OP_VARIANTS`, `DITHER_VARIANTS`). Owns each param's key, label, kind, range, and default. The single source of truth for *what a param is*. Read by any config; never mutated.
- **State** — `rows: signal(Vec<OpRow>)`. The user's current values. `OpRow` is `{ id, inst: OpInstance }`; `id` is the stable key for the keyed `<For/>` (UI-only, kept off `OpInstance` so the instance stays pure data).

Key types (in `op_instance.rs`):

- `OpInstance { tag: String, values: BTreeMap<String, ParamValue> }` — one op in the live pipeline.
- `ParamValue` — `Num(f64)` (covers both Float and Int; the schema's `ParamKind` carries the int/float distinction and the boundary narrows to `u32`/`f32` per field), `Bool(bool)`, `Palette(Vec<String>)`, `Dither(Option<DitherChoice>)`.
- `DitherChoice { tag, values }` — a nested tag+scalar-bag, structurally a mini-instance. Its own type (not reused `OpInstance`) so its bag is provably scalar-only — the recursion is exactly two deep, no deeper.

Why `BTreeMap` not `HashMap`: deterministic iteration, so YAML output is stable.

### Construction and the boundary

- `default_instance(tag)` builds a fresh instance by reading defaults from `OP_VARIANTS`. Defaults live in the schema and nowhere else — this is why adding a scalar op is one table row.
- `OpInstance::to_operation() -> Result<Operation, BuildError>` is the boundary. It reads each key out of the bag, narrows to the field's real type, and assembles the typed `Operation`. It returns `Result` because the bag is `String -> ParamValue` and so a missing/mistyped key is a *runtime* possibility (e.g. an imported YAML predating a schema change) — surfaced as a logged error at Run, not a panic. `DitherChoice::to_config()` does the same for the nested dither enum. Both are hand-written (no serde) so the whole boundary is uniform.

### Edit flow

A config card emits an edit upward via `on_edit: Callback<EditPayload>`, where `EditPayload = (usize, String, ParamValue)` — row id, param key, new value. `PipelineList::edit_op` drops it into that instance's bag with one `values.insert`. That's the entire write path; the serde bridge that used to live here is gone.

Nested (dither) edits stay uniform by committing the *whole* `ParamValue::Dither(Some(choice))` under key `"dither"` — the dither child reads its current choice, mutates a copy, and sends it back as one value. No nested-path message type.

### Config rendering

`op_config_view(id, tag, ...)` dispatches by tag: the five scalar ops go through `generic_op_config`, which loops the variant's params and renders a widget per `ParamKind` (`FloatSlider`/`IntSlider`/`BoolWidget`). `palette_map` is the one hand-written config — its params (`palette`, `dither`) aren't scalars, so it composes its controls directly, reusing `BoolWidget` for the scalar `alpha` param. (`BoolWidget` is shared: the generic loop renders it via a `<BoolWidget/>` tag; palette_map renders it the same way. One checkbox implementation.)

### Other state

- `source: RwSignal<Option<Image>>` — decoded source (`RgbaImage`). Written by the Viewport's file input, read by the run handler.
- `output_url: RwSignal<Option<String>>` — result as a PNG `data:` URL. Written by the run handler, read by the Viewport's `<img>`.
- Palettes load once at startup from `palettes.yaml` (`include_str!`), parsed to a name-sorted `Vec<(String, Vec<String>)>`, provided via context as `StoredValue<Palettes>`, read by the palette-map config.

### The run path

The **Run** button renders in `PipelineList` but the logic lives at `App` root: `App` passes `on_run: Callback<()>` and `can_run: Signal<bool>` (derived from `source`, drives the disabled state). So the child triggers without ever holding `source`/`output_url`. On click:

1. Read the source `RgbaImage` (button disabled while `None`).
2. Map each row's `inst.to_operation()` into a `Vec<Operation>`, short-circuiting on the first `BuildError` (logged, run aborts).
3. `pixelizer_core::apply(&pipeline, image)` runs the ops in order, each consuming the previous image by value.
4. PNG-encode → base64 `data:` URL → `output_url`. Errors logged, not surfaced in UI.
5. Viewport's `<img>` reactively displays it.

`decode` / `encode_to_data_url` (in `viewport.rs`) are plain functions free of Leptos/DOM types — deliberate, so the heavy work can move into a web worker later without dragging UI code along.

### YAML preview

A toggle below Run reveals a live YAML serialization of the current pipeline (via `to_operation()` on each row, then `serde_yaml::to_string`), with copy-to-clipboard. Output is exactly what `cli` parses — copy, save as `.yaml`, runs unmodified through the CLI. Lazy: only serializes while shown, re-runs reactively.

## Dependencies of note

- `leptos` (csr) — reactive UI.
- `leptos-use` — `use_element_size` drives the op card's collapse animation.
- `gloo-file` — async read of uploaded bytes.
- `pixelizer-core` (workspace path) — the pipeline; re-exports `image`, keeping decode/encode on core's `image` version.
- `base64` — the result `data:` URL.
- `serde_yaml` — YAML preview; same crate+version `cli` parses with, so it round-trips. (Deprecated upstream — see ROADMAP for the consolidation plan.)
- `yaml_serde` — parsing `palettes.yaml`. (The second YAML crate the ROADMAP wants to eliminate.)
- `gloo-timers` — the "Copied!" revert delay.
- `wasm-bindgen-futures` — `spawn_local` for the file read and clipboard write.
- `web-sys` — DOM types for the file input and clipboard.
- `console_error_panic_hook` — readable panics in the console.

No `serde_json` — the edit path that needed it is gone.

## Design notes

- **Rust/WASM primary, JS as a thin interop edge.** No application logic in JS.
- **Value-bag state, typed only at the boundary.** The trade: the edit path gives up compile-time field guarantees (the bag is `String -> ParamValue`) in exchange for deleting the entire serde translation layer. The guarantees come back at `to_operation()`, which is *about* to validate anyway. This is the load-bearing decision; see Architecture.
- **Owned-value pipeline.** Each op takes the image by value and returns a new one — matches what they physically do (each allocates, often at new dimensions), so the image is *moved* through the chain, no in-place mutation or cloning in `apply`.
- **State lifted to root.** Run trigger, pipeline, and viewport coordinate only through parent-held signals; children never reach into each other.
