mod dither;
mod generic_config;
mod palette_map;

use generic_config::generic_op_config;
use palette_map::palette_map_config;

use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;
use pixelizer_core::ui_api::ParamKind;
use serde_json::{Map, Value};

pub fn op_config_view(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match op {
        Operation::Posterize { .. }
        | Operation::Normalize { .. }
        | Operation::Downsample { .. }
        | Operation::Upscale { .. }
        | Operation::Blur { .. } => generic_op_config(id, op, rows, on_edit),

        // --- special-case ---
        Operation::PaletteMap { .. } => palette_map_config(id, rows, on_edit),
    }
}

// ----------------------------------------------------------------------------
// Shared, path-agnostic helpers
// ----------------------------------------------------------------------------
// These know nothing about Operation OR DitherConfig — they operate on a plain
// serde_json object. The caller decides *which* object (the op itself, or the
// nested dither enum) to read from / write into. That's the whole reason the
// dither config can reuse the slider machinery: only the path differs.

/// Re-type the f64 a slider hands us into the JSON type the field actually has.
/// This is the one genuinely shared bit between generic ops and dither params.
pub(super) fn typed_value(kind: ParamKind, new_val: f64) -> Value {
    match kind {
        // Keep it a float; core fields are f32 and serde will narrow.
        ParamKind::Float { .. } => serde_json::json!(new_val),
        ParamKind::Int { min, max, .. } => {
            let clamped = (new_val.round() as i64).clamp(min, max);
            serde_json::json!(clamped)
        }
        ParamKind::Bool { .. } => serde_json::json!(new_val != 0.0),
    }
}

/// Read one scalar field as f64 out of an already-serialized object.
/// Bools come back as 1.0/0.0 so a single Signal<f64> path covers every kind.
pub(super) fn read_f64(obj: &Map<String, Value>, key: &str) -> Option<f64> {
    obj.get(key).and_then(|v| match v {
        Value::Number(n) => n.as_f64(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    })
}

/// Read one field as i64 (for Int params). Stays integral.
pub(super) fn read_i64(obj: &Map<String, Value>, key: &str) -> Option<i64> {
    obj.get(key).and_then(Value::as_i64)
}
