use crate::op_card::config::generic_config::sliders::{FloatSlider, IntSlider, decimals_for_step};
use crate::op_card::config::{read_f64, read_i64, typed_value};
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;
use pixelizer_core::ui_api::{ParamKind, VariantDescriptor, dither_variants};
use serde_json::{Map, Value};

// ----------------------------------------------------------------------------
// Reading the nested dither state
// ----------------------------------------------------------------------------

/// Pull the live `Option<DitherConfig>` for this op out of the rows signal,
/// already serialized to a JSON object. `None` means "no dithering".
/// The object, when present, looks like:
///   { "algorithm": "atkinson", "bleed": 1.0, "clamp": false }
fn read_dither_obj(rows: ReadSignal<Vec<OpRow>>, id: usize) -> Option<Map<String, Value>> {
    rows.with(|rs| {
        rs.iter().find(|r| r.id == id).and_then(|r| match &r.op {
            Operation::PaletteMap { dither, .. } => {
                dither.as_ref().and_then(|d| match serde_json::to_value(d) {
                    Ok(Value::Object(m)) => Some(m),
                    _ => None,
                })
            }
            _ => None,
        })
    })
}

/// The current variant tag ("atkinson", "bayer4", …), or None when off.
fn current_tag(rows: ReadSignal<Vec<OpRow>>, id: usize) -> Option<String> {
    read_dither_obj(rows, id).and_then(|m| {
        m.get("algorithm")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

// ----------------------------------------------------------------------------
// Building a fresh DitherConfig from a descriptor's defaults
// ----------------------------------------------------------------------------

/// Construct the JSON object for a dither variant entirely from its descriptor
/// defaults. This is what lets variant-switching stay generic: we never carry
/// fields across (bleed/clamp vs strength don't overlap) — we materialize the
/// new variant clean. Adding a variant to the table needs no change here.
fn default_obj_for(v: &VariantDescriptor) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("algorithm".to_string(), Value::String(v.tag.to_string()));
    for p in v.params {
        let val = match p.kind {
            ParamKind::Float { default, .. } => serde_json::json!(default),
            ParamKind::Int { default, .. } => serde_json::json!(default),
            ParamKind::Bool { default } => serde_json::json!(default),
        };
        m.insert(p.key.to_string(), val);
    }
    m
}

/// Edit closure: set `dither` to the given JSON object (or None to clear it).
/// Round-trips through serde so a malformed object leaves the op untouched
/// rather than corrupting it.
fn set_dither_closure(obj: Option<Map<String, Value>>) -> Box<dyn Fn(&mut Operation)> {
    Box::new(move |op: &mut Operation| {
        if let Operation::PaletteMap { dither, .. } = op {
            match &obj {
                None => *dither = None,
                Some(m) => {
                    if let Ok(cfg) = serde_json::from_value(Value::Object(m.clone())) {
                        *dither = Some(cfg);
                    }
                }
            }
        }
    })
}

/// Edit closure: set ONE param key inside the existing dither object. Reads the
/// live object, overwrites `key` with the typed value, writes it back. If there
/// is no dither yet (toggle off), this is a no-op by construction.
fn set_dither_field_closure(
    rows: ReadSignal<Vec<OpRow>>,
    id: usize,
    key: &'static str,
    kind: ParamKind,
    new_val: f64,
) -> Box<dyn Fn(&mut Operation)> {
    // Capture the current object now; the closure runs against op in place but
    // we need the sibling fields (algorithm + the other param) to ride along.
    let current = read_dither_obj(rows, id);
    Box::new(move |op: &mut Operation| {
        let Operation::PaletteMap { dither, .. } = op else {
            return;
        };
        let Some(mut m) = current.clone() else {
            return;
        };
        m.insert(key.to_string(), typed_value(kind, new_val));
        if let Ok(cfg) = serde_json::from_value(Value::Object(m)) {
            *dither = Some(cfg);
        }
    })
}

// ----------------------------------------------------------------------------
// Param sliders (reuse the generic slider components)
// ----------------------------------------------------------------------------

fn dither_param_widget(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    p: &'static pixelizer_core::ui_api::ParamDescriptor,
) -> AnyView {
    let key = p.key;
    let kind = p.kind;

    match kind {
        ParamKind::Float {
            default,
            min,
            max,
            step,
        } => {
            let value = Signal::derive(move || {
                read_dither_obj(rows, id)
                    .and_then(|m| read_f64(&m, key))
                    .unwrap_or(default as f64)
            });
            let on_commit = Callback::new(move |raw: f64| {
                on_edit.run((id, set_dither_field_closure(rows, id, key, kind, raw)));
            });
            view! {
                <FloatSlider
                    label=p.label
                    value=value
                    min=min as f64 max=max as f64 step=step as f64
                    decimals=decimals_for_step(step as f64)
                    on_commit=on_commit
                />
            }
            .into_any()
        }

        ParamKind::Int { default, min, max } => {
            let value = Signal::derive(move || {
                read_dither_obj(rows, id)
                    .and_then(|m| read_i64(&m, key))
                    .unwrap_or(default)
            });
            let on_commit = Callback::new(move |raw: i64| {
                on_edit.run((
                    id,
                    set_dither_field_closure(rows, id, key, kind, raw as f64),
                ));
            });
            view! {
                <IntSlider
                    label=p.label
                    value=value
                    min=min max=max
                    on_commit=on_commit
                />
            }
            .into_any()
        }

        ParamKind::Bool { default } => {
            let value = Signal::derive(move || {
                read_dither_obj(rows, id)
                    .and_then(|m| read_f64(&m, key))
                    .unwrap_or(if default { 1.0 } else { 0.0 })
            });
            let on_commit = Callback::new(move |raw: f64| {
                on_edit.run((id, set_dither_field_closure(rows, id, key, kind, raw)));
            });
            view! {
                <label class="flex items-center gap-2 text-xs text-slate-400 px-3 pb-2">
                    <input
                        type="checkbox"
                        prop:checked=move || value.get() != 0.0
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            on_commit.run(if checked { 1.0 } else { 0.0 });
                        }
                    />
                    {p.label}
                </label>
            }
            .into_any()
        }
    }
}

// ----------------------------------------------------------------------------
// The component
// ----------------------------------------------------------------------------

/// Dither config, rendered as a child of the PaletteMap card. An on/off toggle
/// controls whether `dither` is Some/None; when on, a variant dropdown plus the
/// selected variant's descriptor params are shown.
#[component]
pub fn DitherConfig(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> impl IntoView {
    // Is dithering currently on? Reactive over the rows signal.
    let enabled = Signal::derive(move || current_tag(rows, id).is_some());

    // First variant in the table is the default when toggled on.
    let on_toggle = move |ev: leptos::ev::Event| {
        let checked = event_target_checked(&ev);
        let obj = if checked {
            dither_variants().first().map(default_obj_for)
        } else {
            None
        };
        on_edit.run((id, set_dither_closure(obj)));
    };

    // Switch variant: materialize the chosen variant fresh from its defaults.
    let on_variant = move |ev: leptos::ev::Event| {
        let tag = event_target_value(&ev);
        if let Some(v) = dither_variants().iter().find(|v| v.tag == tag) {
            on_edit.run((id, set_dither_closure(Some(default_obj_for(v)))));
        }
    };

    // The selected variant's descriptor, to drive the param sliders.
    let selected = Signal::derive(move || {
        current_tag(rows, id).and_then(|tag| dither_variants().iter().find(|v| v.tag == tag))
    });

    view! {
        <div class="border-t border-slate-800 mt-2 pt-2">
            <label class="flex items-center gap-2 text-xs font-bold text-teal-300 px-3 pb-1">
                <input
                    type="checkbox"
                    prop:checked=move || enabled.get()
                    on:change=on_toggle
                />
                "Dither"
            </label>

            {move || enabled.get().then(|| view! {
                <div class="px-3 pb-2">
                    <select
                        class="bg-slate-900 border border-slate-700 rounded-md text-sm \
                               text-slate-200 p-2 w-full"
                        on:change=on_variant
                    >
                        {dither_variants().iter().map(|v| {
                            let tag = v.tag;
                            view! {
                                <option
                                    value=tag
                                    selected=move || current_tag(rows, id).as_deref() == Some(tag)
                                >
                                    {v.label}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </div>

                {move || selected.get().map(|v| {
                    v.params
                        .iter()
                        .map(|p| dither_param_widget(id, rows, on_edit, p))
                        .collect_view()
                })}
            })}
        </div>
    }
}
