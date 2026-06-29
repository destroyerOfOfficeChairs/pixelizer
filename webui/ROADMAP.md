# Roadmap · webui

Planned features and work in progress for the `webui` crate, ordered by value-to-effort. For what already works, see [README.md](README.md).

The single biggest item — moving the pipeline off the main thread — is involved enough to live at the bottom despite its high value; everything above it is cheaper and independently shippable.

---

## In progress

### Palette Map op card — swatch UI

The Palette Map card currently exposes only a `<select>` dropdown of the baked-in named palettes (`palette_map.rs`). The richer swatch-based editor is partially wired: `palette_map_config` already derives the current palette's colors into a `_colors_to_map` signal (presently underscore-prefixed and unused), which is the intended data source for rendering swatches. The remaining build-out, roughly in dependency order:

- **Color swatches** — render the current palette as a row of colored squares (driven by the already-derived colors signal), instead of or alongside the dropdown.
- **Add-swatch affordance** — an empty `+` swatch the user clicks to add a new color.
- **Custom color picker** — clicking a swatch (including the `+`) opens a picker. The native browser picker is inadequate, so this is a from-scratch component.
- **Dithering configuration** — surface the `DitherConfig` options (Floyd–Steinberg, Atkinson, JJN, Bayer4/8, with their `bleed`/`clamp`/`strength` parameters) on this same card. Core's `ui_api` already exposes a `DITHER_VARIANTS` descriptor table parallel to the operation table, so the per-variant parameter widgets can reuse the same descriptor-driven rendering the generic op card uses (`IntSlider`/`FloatSlider`, plus a still-to-be-built bool toggle for `clamp`). What's bespoke here is the layer above the params: an algorithm-picker dropdown that swaps which variant's parameters show, and assembling the chosen variant back into a `DitherConfig` via the same serde bridge. This is the first place a bool parameter actually needs rendering, so it's where a reusable `BoolToggle` component gets extracted.

This is the most feature-dense card and the main remaining UI build-out.

---

## Near-term (cheap, self-contained)

### Viewport: show the source image on upload

Currently nothing is displayed until the pipeline produces an `output_url`. The Viewport should fall back to rendering the decoded `source` image when no output exists yet, so the user sees their image immediately after selecting it. Small change to the Viewport's reactive `<img>` block — read `output_url` first, fall back to a data-URL-encoded `source`.

### Viewport: fit the image within the visible area

Constrain the displayed image so it never overflows the screen regardless of dimensions. Currently only `max-w-full` is applied. Mostly a CSS/Tailwind sizing pass on the Viewport container.

### Clear-image button

A control to reset `source` (and `output_url`) back to `None`, returning the Viewport to its empty state without a page reload.

---

## Medium

### Droppable palette files

Allow dropping a palette file onto the UI so colors don't have to be chosen one at a time. Today the only palettes are the pre-compiled options baked in from `palettes.yaml` via `include_str!`; this would let users supply their own at runtime. Pairs naturally with the swatch UI above (a dropped file populates the swatches).

### Zoom controls in the viewport

Let the user zoom the displayed result — important for inspecting pixel-art output, where the meaningful detail is at the pixel level and the upscaled image may still want closer inspection.

### Drag-and-drop reordering

Make op cards reorderable by dragging, behaving like dnd-kit's sortable (pointer-driven drag with a lifted card and the rest animating out of the way). The current up/down arrow buttons (`on_move` in `pipeline_list.rs`) are the placeholder. The card header bar in `op_card.rs` is already marked as the future drag handle (it has `cursor-grab` and the toggle button already calls `stop_propagation` so a future drag handler won't conflict). The keyed `<For/>` is already in place, so list animation on reorder is partly solved.

### Per-op preview

Let the user click or hover an op card to see what that specific step does — an intermediate preview of the pipeline up to and including that operation. Depends on being able to run sub-pipelines cheaply, so it's more valuable once runs are off the main thread (below).

---

## Large

### Web workers — move the pipeline off the main thread

The pipeline runs on the main thread today, freezing the UI for the length of a run. Moving it to a web worker keeps the UI responsive. A web worker is a separate thread with no shared memory, so this is a request/response restructuring rather than a drop-in swap.

Rough shape:

1. **A worker entry point** — a separate compiled artifact the browser loads independently. It receives a message containing the source image bytes and the pipeline, runs the existing decode → apply → encode chain, and posts the resulting PNG data URL back. The already-isolated, DOM-free helper functions (`decode`, `encode_to_data_url`) move here largely unchanged.
2. **The run handler becomes a send**, not a compute. Instead of calling `apply` inline, it serializes the inputs and posts them to the worker, then returns immediately — this is what removes the freeze.
3. **A result handler** receives the worker's reply and writes `output_url`. The write-to-signal migrates out of the click handler and into this message handler, because the result now arrives asynchronously rather than as a return value.

What crosses the boundary is serialized, so a live `RgbaImage` can't be sent; the plan is to send the original encoded file bytes plus the pipeline and let the worker own the whole decode/apply/encode chain, sending back a string. This means `source` may need to hold (or also retain) the original file bytes, not just the decoded `RgbaImage` — worth deciding early. The [`gloo-worker`](https://docs.rs/gloo-worker/) crate provides a typed request/response abstraction over raw `postMessage` and is the likely path.

This is the fiddliest part of the project — it requires a second build target and getting Trunk to emit and serve both artifacts. High value (it's the headline UX problem), but gated behind real build-system work, hence its place at the bottom.

### Small-screen / responsive layout

The layout is currently desktop-oriented (a fixed two-column flex). Adapting it for phones and narrow windows is a known gap, flagged in the README's run instructions. Lower priority than the functional work above unless mobile use becomes a real target.

### Pipeline import

The YAML preview (with copy-to-clipboard) already covers *export* — the displayed YAML round-trips with the CLI, so copying it and saving as a `.yaml` produces valid CLI input. The missing half is *import*: paste or drop a YAML pipeline and deserialize it back into `rows`. This is the more involved half — it has to rebuild `OpRow`s with fresh ids from a parsed `Pipeline`, and decide how to handle a YAML that fails to parse (surface the error, don't clobber the current pipeline). With both halves, the YAML preview becomes a full export/import surface rather than a read-only debug view.

---

## Workspace-wide (not webui-only)

### Consolidate YAML crates onto a single maintained dependency

> Touches `cli` and `core` docs as well as `webui` — recorded here because the webui's YAML preview is what introduced a third YAML code path and surfaced the inconsistency.

The workspace uses YAML three ways now: `cli` parses pipelines with `serde_yaml` 0.9.34; `webui` parses `palettes.yaml` with `yaml_serde`; and `webui` serializes pipelines for the preview, also with `serde_yaml`. That's two different YAML crates, and `serde_yaml` is deprecated/unmaintained (the author archived it in 2024). It still works — pinning it is a defensible choice for a personal project — but it's tracked tech debt, recorded here so it's a deliberate state rather than an accidental one.

The cleanup: pick one maintained crate and use it everywhere, dropping `yaml_serde`. Candidate is **serde-saphyr** — modern, serde-integrated typed deserialization. Its one notable limitation (no `Value` DOM, so you can't hold a YAML `Value` in flight) doesn't affect this workspace: every YAML site here is typed (`from_str::<Pipeline>`, `from_str::<HashMap<..>>`, `to_string(&pipeline)`), none holds a `Value`. Alternatives if it doesn't fit: noyalib (drop-in `serde_yaml` compat shim that keeps a `Value` type) or yaml-rust2 (lower-level, no serde wrapper).

**The one real risk to verify before committing:** the internally-tagged `DitherConfig` enum (`#[serde(tag = "algorithm")]`) must serialize and round-trip identically under the new crate. Internally-tagged enums are exactly where YAML implementations diverge. Test by serializing a `palette_map` op with each dither variant (especially a Bayer one with `strength` and a diffusion one with `bleed`/`clamp`) and confirming the output parses back through the CLI unchanged — compare against a known-good `default_pipeline.yaml`.

Effort: moderate, low urgency. A deliberate tidying pass, not something to bundle into a feature.
