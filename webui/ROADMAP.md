# Roadmap · webui

Planned work for the `webui` crate, ordered by value-to-effort. For what works, see [README.md](README.md).

The biggest item — moving the pipeline off the main thread — sits at the bottom despite high value: everything above it is cheaper and independently shippable.

---

## In Progress

### The `Inserter` — replace `AddOp`

`AddOp` is a bare `<select>` of op tags. It's the last piece of the core loop that never got a design pass, and it has two problems: ops can only be appended (so "blur *before* downsample" means add-then-reorder), and a list of names communicates nothing about what an op does.

**The pattern.** A thin horizontal insertion point — the WordPress block-inserter idea. Fenceposted through the pipeline: one before the first card, one after every card, so `n` cards get `n+1` inserters. Clicking one opens a menu of operations; picking one inserts *at that position*. This makes position part of the add gesture rather than something to fix afterward with reordering.

Deliberately scoped **out** of this item, in priority order for later:

- **Hover descriptions** (a popup to the side of the menu explaining what each op does). This is the thing that actually fixes op opacity, and it's a separate feature that drops into the menu's row structure once the menu exists. Do it next, not now.
- **Before/after preview thumbnails** in the menu rows. Content work (produce and ship two images per op) more than code work. Slots into the same rows.
- **Categorizing ops.** Six ops don't need categories. Revisit somewhere past a dozen — and only once it's clear which of core's roadmap ops actually get built, since guessing a taxonomy now means committing to one before the data exists.

#### Implementation notes

**1. Move id allocation up first.** `AddOp` currently owns `next_id: StoredValue<usize>` and pushes rows itself. Two components can't both mint ids safely, so before anything else: move the counter into `PipelineList`, and give it an insert write path alongside the existing `move_op` / `remove_op` / `edit_op`.

Take the position as **"before this card id"**, not as an index — see note 3 for why:

```rust
// `before_id: None` means append (the trailing inserter).
let insert_op = move |before_id: Option<usize>, tag: &str| {
    let Some(inst) = default_instance(tag) else { return };
    let id = next_id.get_value();
    next_id.set_value(id + 1);
    set_rows.update(|rows| {
        let pos = before_id
            .and_then(|bid| rows.iter().position(|r| r.id == bid))
            .unwrap_or(rows.len());
        rows.insert(pos, OpRow { id, inst });
    });
};
```

Resolving the position *inside* the `update` closure means it's computed against the current list at the moment of insertion — nothing cached, nothing that can go stale. `unwrap_or(rows.len())` covers both the trailing inserter (`None`) and the shouldn't-happen case of an id that's no longer present. Note `App` seeds `rows` with a hardcoded `id: 0`, so `next_id` starts at 1 — same as `AddOp` does today.

**2. Inserters are siblings of cards, not children of them.** Tempting shape: put an `Inserter` at the bottom of `OpCard`. Two reasons it's wrong — the "insert above the first card" affordance has no card to live in (needing a special case is the tell that the pattern doesn't fit), and `OpCard` would have to carry an `on_insert` prop that has nothing to do with rendering a card. Flat and alternating, all children of `PipelineList`:

```
PipelineList
├── Inserter (before card 0)
├── OpCard   (id 0)
├── Inserter (before card 1)
├── OpCard   (id 1)
├── Inserter (trailing — append)
└── ...
```

`OpCard` is unchanged and unaware inserters exist. `Inserter` knows which card it sits above and a callback, nothing else. Same separation as `ColorPicker` not knowing about `owned`, or `PaletteDropZone` not knowing about `Palettes`.

**3. Inserters aren't separately keyed — they ride inside their row's view.** Inside the `<For/>`, emit the inserter *then* the card; emit the trailing inserter after the loop closes. That yields `n+1` naturally.

```rust
<For
    each=move || rows.get()
    key=|r| r.id
    children=move |r| view! {
        <Inserter before_id=Some(r.id) ... />
        <OpCard id=r.id ... />
    }
/>
<Inserter before_id=None ... />   // trailing; outside the For
```

There is only **one key space** — row ids. The inserter is part of a row's output, so it needs no key of its own, and the "index vs. id" question dissolves.

An inserter identifies its slot as **"before card X"**, never as a stored index. That relationship is relative, so it survives reordering for free: the inserter rendered alongside card A is always immediately above A, wherever A ends up. Reading top to bottom the gaps are still 0, 1, 2 — the DOM node moved, but the slot it represents is correct at its new location. A *stored* index would break here: after a reorder you'd have an inserter labelled "insert at 1" sitting in the position-0 gap.

The first inserter changes owner across a reorder (it was A's, now it's B's) — different node, same job, no special case. The trailing inserter never moves; it's outside the `<For/>`.

Concrete index resolution happens once, at click time, inside `insert_op` (note 1). Nothing is cached, so nothing can drift.

**4. Visual noise is the main design risk.** Six ops means seven dividers; if each is a persistent line the list becomes striped and the cards lose prominence. Standard mitigation: the inserter is a mostly-invisible hover zone that reveals a line and a `+` on hover — the same `group-hover` trick the swatch `×` uses. Exception: keep the **trailing** inserter always visible, so there's one obvious add affordance that teaches the pattern and preserves the current "add at the end" mental model.

**5. The menu is the palette dropdown pattern again.** Trigger captures its own rect, panel portals to `<body>` (the pipeline column will clip otherwise), `position: fixed`, `on_click_outside` to dismiss, close on scroll and resize. Rows come from `all_op_menu()` — `(tag, label)` pairs straight from `OP_VARIANTS`, same source `AddOp` uses now.

**6. Positioning is genuinely harder here than for the palette dropdown.** That one has a single instance near the top of a card and only *just* fits below without flipping. The inserter has `n+1` instances scattered down a scrolling column, so both top and bottom overflow are real. Two pieces:

- **Cap the menu height** (~`320px`) with internal scrolling, so a growing op list can't produce a menu taller than any viewport.
- **Flip when it doesn't fit below.** The picker's approach (a measured height constant, in `swatches.rs`) works because the picker's content is fixed. The menu's height is data-dependent, so prefer anchoring by CSS `bottom` when flipping — set `bottom` from the trigger's *top* edge and let the browser compute the height, instead of guessing it.

**7. Delete `AddOp`** once the trailing inserter covers it. Keeping both would be the redundancy this item exists to remove — the last fencepost *is* the bottom add button.

#### Done when

- Every gap between cards, plus above the first and below the last, can insert an op at that position.
- The menu opens near its trigger, flips rather than overflowing at both the top and bottom of the viewport, and scrolls internally when the op list is long.
- Dismisses on outside click, scroll, and resize.
- `add_op.rs` is deleted and `next_id` lives in `PipelineList`.

#### How this interacts with drag reordering

Building the Inserter first is deliberate sequencing: it's the smaller piece, and it leaves scaffolding the bigger piece wants (the `insert_op(before_id, …)` write path is close to the `move_op(from, to)` drag needs; the `dragging` signal below is needed either way).

The two features coexist — WordPress and Notion both ship both — but they touch:

- **Inserters move with their card, and that's correct.** Each leading inserter is part of its row's view (note 3), so reordering carries it along — the DOM node for "before A" travels with A. The *sequence* of gaps stays right regardless: reading top to bottom you still get gap 0, gap 1, gap 2. Nothing needs to hold inserters in place, and only cards need FLIP transforms — the inserter inside a row is transformed along with it. If inserters have real height, the distance a card travels includes the inserters it passes; FLIP measures actual before/after positions, so that's handled automatically. The trailing inserter is outside the `<For/>` and never moves.
- **Apply the drag transform to the card element, not the row wrapper.** Each `<For/>` child is an inserter plus a card, so transforming the row would lift both — a thin strip of inserter riding above the card. Put the transform on the `<OpCard>` element instead and the lifted thing is the card alone, which is what dnd-kit does. No extra cost: there's a `NodeRef` on the card for measuring anyway. Mostly cosmetic in practice, since the `dragging` signal hides the inserters' `+` during a drag regardless.
- **A `dragging` signal suppresses the add-affordance.** During a drag the pointer crosses inserters constantly, and `+` buttons flickering in and out looks broken. One signal (`RwSignal<bool>`, or `Option<usize>` for *which* card) read by every inserter: hide the `+`, switch to a drop-indicator style.
- **`pointer-events: none` on the dragged card.** Otherwise the card following the cursor sits between the pointer and the inserter underneath, and the inserter never sees a hover. Same class already used on the color picker's handles.
- **The inserters adjacent to the dragged card are no-ops** (dropping there means "leave it where it is"). Dimming them avoids offering a meaningless target. Nice-to-have.

One correction to a tempting assumption: inserters are a good place to *render* the drop indicator, but probably not what to *hit-test* against. dnd-kit computes the target index from card **midpoints** — target changes when the pointer crosses a card's center. That's smoother than testing against gaps, which are thin (a few px) next to cards (~50px+). So: card-midpoint math for the hit test, inserters for the visual.

Suggested staging when the time comes: **drag with instant snapping first** (pick up, drop, list reorders, no animation) to prove pointer tracking, hit testing, and the reorder write path. Then add FLIP as a pure enhancement. If FLIP fights Leptos harder than expected — and it might, since `<For/>` owns the nodes and may recreate them on reorder, so the "before" and "after" elements measured aren't guaranteed to be the same DOM nodes — working drag-and-drop still exists rather than nothing.

---

## Later Goals

### Palette file download

Allow the user to download any custom palette they've created.

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

### Drag-and-drop reordering

Make op cards reorderable by dragging (dnd-kit-sortable behavior: pointer drag, lifted card, the rest animating aside). The up/down arrows (`on_move` in `pipeline_list.rs`) are the placeholder — they go away once this lands. The card header in `op_card.rs` is already the intended drag handle — it has `cursor-grab`, and the collapse toggle already calls `stop_propagation` so a future drag handler won't conflict. The keyed `<For/>` means reorder animation is partly solved.

Blocked on the Inserter by choice, not necessity — see **How this interacts with drag reordering** under In Progress for the `dragging` signal, why inserters riding along with their cards is correct, hit-testing against card midpoints rather than gaps, and the suggested snap-first-then-FLIP staging.

Note that once insert-at-position exists, `move_op` is no longer the *only* way to get an op somewhere other than the end — which lowers the urgency of this item without lowering its value.

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

Further expanding on the `pipeline import` goal, create a custom file type (Or use some existing file type that makes sense for this purpose) to save an entire workflow, images included.

This will need a way to save/load files, and the associated UI.

### `Normalize` operation - optimization

The histogram only needs to be calculated once, provided the previous operations don't change.

May need to alter the core module for this.
