# Roadmap · webui

Planned work for the `webui` crate, ordered by value-to-effort. For what works, see [README.md](README.md).

The biggest item — moving the pipeline off the main thread — sits at the bottom despite high value: everything above it is cheaper and independently shippable.

---

## In progress

### Palette Map card — palette selector

**Replace the `<select>` with a custom picker component.** A native `<option>` renders text only — no per-row swatches, no arbitrary entries, no "Custom" state, no embedded drop zone.

**In-list swatches** — each picker row shows the palette name plus a mini swatch strip, so all options are visible without click-testing each. This is *why* the native select had to go; it's the first payoff of the picker.

**File upload → new named entry.** Two separable problems, different difficulty:
  - *Intake:* start with `<input type="file">` (~10 lines, reuses the async-read pattern in `viewport.rs` — `gloo_file` + `read_as_bytes`). A drop zone is nicer but adds drag-event handling and visual states; do it second, once parsing works. The picker can host the drop zone once it exists.
  - *Parsing:* format difficulty varies wildly (see the format list below). An uploaded file's name becomes a new entry in the picker.

**Palette file formats** (from lospec's export options), easiest-first — implement the cheap text ones, skip the binary one:

- **HEX** — one hex code per line. Trivial; maps straight onto the existing `Vec<String>` of `#rrggbb`. *Do first.*
- **GIMP GPL** — text; `R G B Name` rows after a header line. Easy.
- **JASC PAL** — text; a count line then `R G B` rows. Easy.
- **Paint.NET TXT** — one `AARRGGBB` hex per line, `;` comments. Easy (strip the alpha).

HEX + GPL covers the two most common lospec exports for almost no cost; the other two text formats are cheap add-ons.

**File download**

Allow the user to download any custom palette they've created.

---

## Later Goals

### Color picker bug

The color picker appears off-screen if a swatch is too low on the screen.

### Viewport polish

- Clear image button
- Zoom controls
- Fit exactly to screen
- Make the pipeline list the only place the user can scroll. Viewport is fixed.
- Image translation (left/right/up/down in the viewport)
- Add resolution text in a small bottom-bar (pixel-native & actual resolution)
- Fix bug where uploading a new image (after one has already been loaded) does not show in the viewport.

### Undo/Redo buttons

Probably more work than it seems, but it would be worth it.

### Droppable palette files

Let users drop a palette file so colors aren't chosen one at a time. Today palettes are only the compiled-in options from `palettes.yaml` (`include_str!`); this supplies them at runtime. Pairs with the swatch UI — a dropped file populates the swatches.

### Drag-and-drop reordering

Make op cards reorderable by dragging (dnd-kit-sortable behavior: pointer drag, lifted card, the rest animating aside). The up/down arrows (`on_move` in `pipeline_list.rs`) are the placeholder. The card header in `op_card.rs` is already the intended drag handle — it has `cursor-grab`, and the collapse toggle already calls `stop_propagation` so a future drag handler won't conflict. The keyed `<For/>` means reorder animation is partly solved.

### Per-op preview

Click/hover an op card to preview the pipeline up to and including that op. Depends on running sub-pipelines cheaply, so it's more valuable once runs are off the main thread (below).

---

## Goals With Huge Scope And High Workload

### Web workers — move the pipeline off the main thread

The pipeline runs on the main thread, freezing the UI for a run's duration. A worker fixes it — but a worker is a separate thread with no shared memory, so this is a request/response restructuring, not a drop-in swap.

Shape:

1. **Worker entry point** — a separately-compiled artifact the browser loads. Receives source image bytes + the pipeline, runs decode → apply → encode, posts the PNG data URL back. The DOM-free helpers (`decode`, `encode_to_data_url`) move here largely unchanged — they were kept Leptos-free for exactly this.
2. **Run handler becomes a send**, not a compute: serialize inputs, post to the worker, return immediately. This is what removes the freeze.
3. **Result handler** receives the reply and writes `output_url`. The signal write migrates out of the click handler into this message handler, because the result now arrives asynchronously.

What crosses the boundary is serialized, so a live `RgbaImage` can't be sent. Plan: send the original encoded file bytes + the pipeline, let the worker own the whole decode/apply/encode chain, get back a string. **Decide early:** `source` may need to hold (or also retain) the original file bytes, not just the decoded `RgbaImage`. [`gloo-worker`](https://docs.rs/gloo-worker/) gives a typed request/response layer over raw `postMessage` and is the likely path.

The pipeline crosses this boundary as a `Vec<Operation>` (already `Serialize`), so serialization is solved — the fiddly part is the build system: a second target, and Trunk emitting and serving both artifacts. High value (the headline UX problem), gated behind real build work — hence bottom.

### Pipeline import

The YAML preview covers *export* — the displayed YAML round-trips with the CLI. Missing half is *import*: paste/drop a YAML pipeline and deserialize back into `rows`. More involved: it must parse a `Pipeline`, then rebuild each `OpRow` — and here's the wrinkle the value-bag introduces, a `Pipeline` holds typed `Operation`s, but `rows` holds `OpInstance` bags, so import needs the *inverse* of `to_operation()`: an `Operation -> OpInstance` conversion that doesn't exist yet. Plus fresh ids, and a decision on parse failure (surface the error, don't clobber the current pipeline). With both halves, the preview becomes a full export/import surface.

### Custom Save File

Further expanding on the `pipeline import` goal, create a custom file type to save an entire workflow, images included.

This will need a way to save/load files, and the associated UI.

### `Normalize` operation - optimization

The histogram only needs to be calculated once, provided the previous operations don't change.

May need to alter the core module for this.
