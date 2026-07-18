# Architecture

A walk from the schema down to a running pipeline, framed around the design decision that shaped everything: the **value-bag**. Once that clicks, the rest falls into place.

## The central tension

The `core` crate has a typed enum, `Operation`, that describes every possible thing the pipeline can do:

```rust
enum Operation {
    Downsample { pixel_size: u32 },
    Blur { sigma: f32 },
    PaletteMap { colors: Vec<String>, dither: Option<DitherConfig>, ... },
    // ...
}
```

This enum is *perfect* for the pipeline runner in `apply()` — it match-arms over each variant, calls the right function, image goes in one side and out the other. Every field is typed, every case is exhaustive, the compiler holds your hand.

But the UI has a different job. The UI needs to render a slider for `pixel_size`, another slider for `sigma`, a checkbox for `preserve_alpha`. And when the user drags a slider, the UI needs to *write back* into whichever field of whichever variant of the enum is currently being edited.

A generic slider component doesn't know that "the value it's editing" is `Operation::Blur::sigma`. So under a typed-first design, you'd need either five bespoke config components (one per op) or a serde bridge that lets a generic slider read/write "the field named `sigma` of the currently-selected op." An earlier version of the codebase actually did the second — serialize the op to JSON, poke a field, deserialize back. It worked, but every edit paid a full serde round-trip.

**The value-bag is the alternative:** don't use the typed enum as the live UI state at all. Use a shape the UI can naturally read and write, and reconstruct the typed enum only at the moment you need it.

## The core side

`core/src/op_schema.rs` describes each op *as data*: name, params, each param's kind and range and default. This is separate from `Operation` the enum — the enum is the runtime type; the schema is metadata *about* the enum.

The schema types are small:

- `ParamDescriptor` — one param: its `key` (the field name), `label` (human string), and `kind`.
- `ParamKind` — an enum of what a param *is*: `Float { default, min, max, step }`, `Int { ... }`, `Bool { default }`, `Palette { colors }`, `Dither { default_tag }`. This is where widget-relevant metadata lives — the min/max/step a slider needs, which the type system alone can't express.
- `VariantDescriptor` — one op or dither variant: its `tag`, `label`, and `params` (a slice of `ParamDescriptor`).

Then two big `const` tables (in `tables.rs`):

- `OP_VARIANTS` — one entry per operation. Downsample has one param, palette_map has three, and so on. Every field name, every default, every slider range is here.
- `DITHER_VARIANTS` — same shape, one entry per dither algorithm.

These tables are the **single source of truth for what a param is.** The types file (`op_schema.rs`) says what shape the truth takes; the tables file has the actual data; and `labels.rs` has small helpers (`label_for_tag`, `all_op_menu`) that answer human-string questions.

A test guards the one place strings could drift silently: `dither_default_tags_exist` checks that every `ParamKind::Dither { default_tag }` names a real entry in `DITHER_VARIANTS`. Typo it and `cargo test` fails.

The rest of `core` is unchanged from a design perspective — the `Operation` enum, `apply()`, the actual image operations. `op_schema` is *additional* metadata riding alongside the runtime types, not a replacement for them.

## The webui side

Now the value-bag itself. In `op_instance.rs`:

```rust
pub struct OpInstance {
    pub tag: String,                                 // "blur"
    pub values: BTreeMap<String, ParamValue>,        // "sigma" → Num(1.0)
}

pub enum ParamValue {
    Num(f64),
    Bool(bool),
    Palette(Vec<String>),
    Dither(Option<DitherChoice>),
}
```

An op instance is a tag (which op it is) plus a map from field name to value. Every widget in the UI reads and writes `values[key]` — the slider for sigma does `values.get("sigma")` and, on drag, `values.insert("sigma", Num(new_value))`. No serde. No knowledge of which typed variant this is. Just: read a key, write a key.

The four arms of `ParamValue` map to the five `ParamKind`s: `Float` and `Int` both fold into `Num(f64)` (the schema carries which is which), `Bool` becomes `Bool`, `Palette` becomes `Palette`, `Dither` becomes `Dither` (nesting a mini-instance for the chosen algorithm and its params).

The webui has three source-of-truth signals at the root (in `main.rs` / `App`):

- `rows: signal(Vec<OpRow>)` — the pipeline. `OpRow` wraps `OpInstance` with a stable UI-only `id` for the keyed `<For/>`.
- `source: RwSignal<Option<Image>>` — the decoded input image.
- `output_url: RwSignal<Option<String>>` — the output as a data URL.

The edit path is one closure, in `pipeline_list.rs`:

```rust
let edit_op = Callback::new(move |(id, key, value): EditPayload| {
    set_rows.update(|rows| {
        if let Some(r) = rows.iter_mut().find(|r| r.id == id) {
            r.inst.values.insert(key, value);
        }
    });
});
```

That's the *entire write path*. Every config card, every widget, every edit funnels through this one `values.insert`. Even nested things like dither commit the *whole* `ParamValue::Dither(Some(choice))` under the key `"dither"` — no nested path type, no recursive edit machinery.

## Widget generic-ness

Because the bag matches the schema shape, one config component handles all five scalar ops. `generic_config.rs` walks `OP_VARIANTS`, and for each param it renders a slider or checkbox by dispatching on the schema's `ParamKind`:

```rust
for param in variant.params {
    match param.kind {
        ParamKind::Float { .. } => <FloatSlider .../>,
        ParamKind::Int { .. } => <IntSlider .../>,
        ParamKind::Bool { .. } => <BoolWidget .../>,
        // ...
    }
}
```

Adding a sixth scalar op is one row in `OP_VARIANTS`. That's the payoff.

`palette_map` is the exception — it has non-scalar params (a palette, a dither), so its config is hand-written in `palette_map.rs`. But even *it* reuses `BoolWidget` for the scalar `alpha` param. One checkbox implementation for both paths.

Similarly, `add_op.rs` is tiny because `all_op_menu()` (schema) and `default_instance(tag)` (bag construction from schema) do the work. The dropdown of "operations you can add" is derived from `OP_VARIANTS`; the fresh instance you push into `rows` is built from the schema's defaults.

## The boundary — the one place types come back

`op_instance/boundary.rs` is where the bag becomes typed again. Every time the user hits Run, each `OpInstance` calls `to_operation()`, which hand-matches on `self.tag.as_str()` and reads each key out of the bag:

```rust
"blur" => Operation::Blur { sigma: self.f32_field("sigma")? },
```

`f32_field` looks up the key, expects a `Num`, narrows to `f32`. Returns `Result` because the bag *could* have a missing or wrong-typed key (imagine importing a YAML pipeline that predates a schema change). A malformed bag surfaces as a logged error at Run, not a panic.

This is the *only place* the schema-vs-bag contract is checked. If a key is missing, if a `ParamValue::Bool` shows up where a `Num` was expected, if the tag is unknown — this is where it fails. Everywhere else in the UI, the bag is just a `BTreeMap<String, ParamValue>` that you read and write freely.

That failure surface being *singular* is the point. Under a typed-first design, every widget-to-op interaction was a potential failure point (the serde round-trip could go wrong anywhere). Under the value-bag, every widget interaction is just a map operation, and the one place things *can* fail is the one place you already needed a boundary — the transition from "the user is editing this" to "the runtime needs to consume this."

## The run path

When the Run button in `pipeline_list.rs` fires:

1. Its `on_click` calls the `on_run` callback (passed down from `App`).
2. `App`'s `on_run` reads `source.get()`, then maps each row's `inst.to_operation()` — that's the boundary — collecting into `Vec<Operation>` or short-circuiting on the first `BuildError` (logged, run aborts).
3. Builds a `Pipeline { operations }` and calls `pixelizer_core::apply(&pipeline, img)`. This runs synchronously on the main thread (moving it to a web worker is the biggest open ROADMAP item).
4. PNG-encodes the result, sets `output_url`.
5. The viewport's `<img>` reactively displays the data URL.

The Run button lives in `PipelineList` (the child) but the *logic* lives in `App` (the root). `PipelineList` gets a `Callback<()>` to trigger the run and a `Signal<bool>` for the disabled state. The child never holds `source` or `output_url`; it just triggers.

## Why this all coheres

The whole design turns on one substitution: instead of the UI state being *typed but generic access is expensive*, the UI state is *stringly-keyed and generic access is free, but you check the shape once at the boundary*.

That substitution is only good because the schema table (`OP_VARIANTS`) exists to *describe* the bag — so widgets aren't flying blind, they're driven by the same table that the boundary reader is going to validate against. The schema is the contract; the bag is the storage; the boundary is the enforcement.

The three-file splits make each of those parts findable on disk: `op_schema/tables.rs` is the contract, `op_instance.rs` is the storage type, `op_instance/boundary.rs` is the enforcement. If you come back to this in six months and want to know "where does the runtime check happen," the filename answers. If you want to know "what params does an op have," different filename. If you want to know "what shape can a value take," a third.

The payoff, again: 5 scalar ops share 1 config component. Adding a 6th is one table row. Adding a new dither algorithm is one table row. The one non-scalar op (`palette_map`) is bespoke, but that bespokeness is *localized* — it doesn't push its shape onto anything else.

That's the architecture, top to bottom.
