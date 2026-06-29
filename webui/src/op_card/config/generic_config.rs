mod sliders;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;
use pixelizer_core::ui_api::{ParamDescriptor, ParamKind, VariantDescriptor, op_variants};
use serde_json::Value;
use sliders::{FloatSlider, IntSlider, decimals_for_step};

/// Read one field as i64 (for Int params). Mirrors read_field but stays integral.
fn read_field_i64(op: &Operation, key: &str) -> Option<i64> {
    match serde_json::to_value(op) {
        Ok(Value::Object(m)) => m.get(key).and_then(Value::as_i64),
        _ => None,
    }
}

/// Find the descriptor row for a given op tag (e.g. "blur").
fn variant_for(tag: &str) -> Option<&'static VariantDescriptor> {
    op_variants().iter().find(|v| v.tag == tag)
}

/// The serde tag string for an Operation instance, via its Serialize impl.
/// (Operation has tag = "type", rename_all = "snake_case".)
fn op_tag(op: &Operation) -> String {
    match serde_json::to_value(op) {
        Ok(Value::Object(m)) => m
            .get("type")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Read one scalar field's current value out of the live Operation, by key.
/// Returns f64 for the slider regardless of the field's real type.
fn read_field(op: &Operation, key: &str) -> Option<f64> {
    match serde_json::to_value(op) {
        Ok(Value::Object(m)) => m.get(key).and_then(|v| match v {
            Value::Number(n) => n.as_f64(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }),
        _ => None,
    }
}

/// Build an edit closure that sets ONE key on the op to `new_val`, going
/// through serde so we don't hand-write a per-field match. The closure:
///   1. serializes the op to a JSON object,
///   2. overwrites obj[key] with the typed value,
///   3. deserializes back into Operation, and
///   4. if that succeeded, overwrites *op in place.
/// Untouched keys (colors, dither, the OTHER scalar field) ride along for free
/// because they were in the serialized object already.
fn set_field_closure(
    key: &'static str,
    kind: ParamKind,
    new_val: f64,
) -> Box<dyn Fn(&mut Operation)> {
    Box::new(move |op: &mut Operation| {
        let Ok(Value::Object(mut m)) = serde_json::to_value(&*op) else {
            return;
        };
        // Re-type the f64 from the slider into the field's real JSON type.
        let typed: Value = match kind {
            ParamKind::Float { .. } => {
                // Keep it a float; core fields are f32 and serde will narrow.
                serde_json::json!(new_val)
            }
            ParamKind::Int { min, max, .. } => {
                let clamped = (new_val.round() as i64).clamp(min, max);
                serde_json::json!(clamped)
            }
            ParamKind::Bool { .. } => serde_json::json!(new_val != 0.0),
        };
        m.insert(key.to_string(), typed);
        if let Ok(new_op) = serde_json::from_value::<Operation>(Value::Object(m)) {
            *op = new_op;
        }
        // If from_value failed (shouldn't, for a valid single-field change),
        // we leave op untouched rather than corrupt it.
    })
}

fn param_widget(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    p: &'static ParamDescriptor,
) -> AnyView {
    let key = p.key; // &'static str: Copy
    let kind = p.kind; // ParamKind: Copy (after the derive you added)

    match kind {
        // -------- Float --------
        ParamKind::Float {
            default,
            min,
            max,
            step,
        } => {
            let value = Signal::derive(move || {
                rows.with(|rs| {
                    rs.iter()
                        .find(|r| r.id == id)
                        .and_then(|r| read_field(&r.op, key))
                })
                .unwrap_or(default as f64)
            });

            let on_commit = Callback::new(move |raw: f64| {
                on_edit.run((id, set_field_closure(key, kind, raw)));
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

        // -------- Int --------
        ParamKind::Int { default, min, max } => {
            let value = Signal::derive(move || {
                rows.with(|rs| {
                    rs.iter()
                        .find(|r| r.id == id)
                        .and_then(|r| read_field_i64(&r.op, key))
                })
                .unwrap_or(default)
            });

            // set_field_closure still takes f64; pass the i64 through as f64.
            // It re-clamps and rounds to i64 internally for Int kinds, so this
            // is lossless for the integer range we care about.
            let on_commit = Callback::new(move |raw: i64| {
                on_edit.run((id, set_field_closure(key, kind, raw as f64)));
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

        // -------- Bool (inline; extract BoolToggle when dither needs it) --------
        ParamKind::Bool { default } => {
            let value = Signal::derive(move || {
                rows.with(|rs| {
                    rs.iter()
                        .find(|r| r.id == id)
                        .and_then(|r| read_field(&r.op, key))
                })
                .unwrap_or(if default { 1.0 } else { 0.0 })
            });

            let on_commit = Callback::new(move |raw: f64| {
                on_edit.run((id, set_field_closure(key, kind, raw)));
            });

            view! {
                <label class="flex items-center gap-2 text-xs text-slate-400 p-3">
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
/// The generic config view: render every descriptor param for this op's tag.
/// Drop-in replacement for the per-op *_config functions, for scalar-only ops.
pub fn generic_op_config(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let tag = op_tag(op);
    let Some(variant) = variant_for(&tag) else {
        return view! { <p class="text-xs text-red-400 p-3">"No descriptor for "{tag}</p> }
            .into_any();
    };

    variant
        .params
        .iter()
        .map(|p| param_widget(id, rows, on_edit, p))
        .collect_view()
        .into_any()
}
