mod blur;
mod number_slider;
mod posterize;

use crate::{EditPayload, OpRow};
use blur::blur_config;
use leptos::prelude::*;
use pixelizer_core::Operation;
use posterize::posterize_config;

pub fn op_config_view(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match op {
        Operation::Blur { .. } => blur_config(id, rows, on_edit),
        Operation::Posterize { .. } => posterize_config(id, rows, on_edit),
        _ => view! {
            <p class="text-xs text-slate-600 italic">"No editable parameters yet."</p>
        }
        .into_any(),
    }
}
