# Roadmap · webui

Planned work for the `webui` crate, ordered by value-to-effort. For what works, see [README.md](README.md).

The biggest item — moving the pipeline off the main thread — sits at the bottom despite high value: everything above it is cheaper and independently shippable.

---

## In progress

### Palette Map card — swatch UI

The dithering config and the `preserve alpha` toggle are **done**; what remains on this card is the palette editor itself. Today it's a `<select>` of baked-in named palettes. `palette_map_config` already derives the current palette's colors into a `_colors` signal (underscore-prefixed, presently unused) — the intended source for swatches. Remaining, in dependency order:

- **Color swatches** — render the current palette as a row of colored squares (driven by `_colors`), instead of or alongside the dropdown.
- **Add-swatch affordance** — an empty `+` swatch to add a color.
- **Custom color picker** — clicking a swatch (or `+`) opens a picker. The native browser picker is inadequate, so this is a from-scratch component.

This is the last real UI build-out on the card. Note the wiring pattern it must follow: a color edit commits `ParamValue::Palette(colors)` under key `"palette"` (see the existing `on_change` handler), same as everything else — the swatch UI changes how colors are *chosen*, not how they're *stored*.

---

## Near-term (cheap, self-contained)

### Viewport: fit the image within the visible area

Constrain the displayed image so it never overflows regardless of dimensions. Only `max-w-full` is applied now — mostly a Tailwind sizing pass on the Viewport container. (Source-on-upload display is already done.)

### Clear-image button

A control to reset `source` and `output_url` to `None`, returning the Viewport to empty without a reload.

---

## Medium

### Droppable palette files

Let users drop a palette file so colors aren't chosen one at a time. Today palettes are only the compiled-in options from `palettes.yaml` (`include_str!`); this supplies them at runtime. Pairs with the swatch UI — a dropped file populates the swatches.

### Zoom controls in the viewport

Zoom the result — important for pixel-art output where the meaningful detail is per-pixel and the upscaled image may still want inspection.

### Drag-and-drop reordering

Make op cards reorderable by dragging (dnd-kit-sortable behavior: pointer drag, lifted card, the rest animating aside). The up/down arrows (`on_move` in `pipeline_list.rs`) are the placeholder. The card header in `op_card.rs` is already the intended drag handle — it has `cursor-grab`, and the collapse toggle already calls `stop_propagation` so a future drag handler won't conflict. The keyed `<For/>` means reorder animation is partly solved.

### Per-op preview

Click/hover an op card to preview the pipeline up to and including that op. Depends on running sub-pipelines cheaply, so it's more valuable once runs are off the main thread (below).

---

## Large

### Web workers — move the pipeline off the main thread

The pipeline runs on the main thread, freezing the UI for a run's duration. A worker fixes it — but a worker is a separate thread with no shared memory, so this is a request/response restructuring, not a drop-in swap.

Shape:

1. **Worker entry point** — a separately-compiled artifact the browser loads. Receives source image bytes + the pipeline, runs decode → apply → encode, posts the PNG data URL back. The DOM-free helpers (`decode`, `encode_to_data_url`) move here largely unchanged — they were kept Leptos-free for exactly this.
2. **Run handler becomes a send**, not a compute: serialize inputs, post to the worker, return immediately. This is what removes the freeze.
3. **Result handler** receives the reply and writes `output_url`. The signal write migrates out of the click handler into this message handler, because the result now arrives asynchronously.

What crosses the boundary is serialized, so a live `RgbaImage` can't be sent. Plan: send the original encoded file bytes + the pipeline, let the worker own the whole decode/apply/encode chain, get back a string. **Decide early:** `source` may need to hold (or also retain) the original file bytes, not just the decoded `RgbaImage`. [`gloo-worker`](https://docs.rs/gloo-worker/) gives a typed request/response layer over raw `postMessage` and is the likely path.

The pipeline crosses this boundary as a `Vec<Operation>` (already `Serialize`), so serialization is solved — the fiddly part is the build system: a second target, and Trunk emitting and serving both artifacts. High value (the headline UX problem), gated behind real build work — hence bottom.

### Small-screen / responsive layout

Layout is a fixed two-column flex, desktop-oriented. Adapting for phones/narrow windows is a known gap (flagged in the README). Lower priority than functional work unless mobile becomes a real target.

### Pipeline import

The YAML preview covers *export* — the displayed YAML round-trips with the CLI. Missing half is *import*: paste/drop a YAML pipeline and deserialize back into `rows`. More involved: it must parse a `Pipeline`, then rebuild each `OpRow` — and here's the wrinkle the value-bag introduces, a `Pipeline` holds typed `Operation`s, but `rows` holds `OpInstance` bags, so import needs the *inverse* of `to_operation()`: an `Operation -> OpInstance` conversion that doesn't exist yet. Plus fresh ids, and a decision on parse failure (surface the error, don't clobber the current pipeline). With both halves, the preview becomes a full export/import surface.

---

## Workspace-wide (not webui-only)

### Consolidate YAML crates onto a single maintained dependency

> Touches `cli` and `core` too — recorded here because the webui's YAML preview introduced the third YAML path that surfaced the inconsistency.

Three YAML uses now: `cli` parses pipelines with `serde_yaml` 0.9.34; `webui` parses `palettes.yaml` with `yaml_serde`; `webui` serializes pipelines for the preview, again `serde_yaml`. Two crates, and `serde_yaml` is archived/unmaintained (2024). It works — pinning it is defensible for a personal project — but it's tracked debt, recorded so it's deliberate rather than accidental.

Cleanup: one maintained crate everywhere, drop `yaml_serde`. Candidate **serde-saphyr** — modern, serde-integrated typed deserialization. Its one limit (no `Value` DOM) doesn't bite here: every site is typed (`from_str::<Pipeline>`, `from_str::<HashMap<..>>`, `to_string(&pipeline)`), none holds a `Value`. Fallbacks: noyalib (drop-in `serde_yaml` shim keeping a `Value`) or yaml-rust2 (lower-level, no serde wrapper).

**Verify before committing:** the internally-tagged `DitherConfig` (`#[serde(tag = "algorithm")]`) must round-trip identically under the new crate — internally-tagged enums are exactly where YAML libraries diverge. Test by serializing a `palette_map` op with each dither variant (a Bayer one with `strength`, a diffusion one with `bleed`/`clamp`) and confirming it parses back through the CLI unchanged against a known-good `default_pipeline.yaml`.

Moderate effort, low urgency. A deliberate tidying pass, not something to bundle into a feature.
