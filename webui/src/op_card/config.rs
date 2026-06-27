mod blur;
mod downsample;
mod normalize;
mod number_slider;
mod palette_map;
mod posterize;
mod upscale;

use blur::blur_config;
use downsample::downsample_config;
use normalize::normalize_config;
use palette_map::palette_map_config;
use posterize::posterize_config;
use upscale::upscale_config;

use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

pub fn op_config_view(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match op {
        Operation::Blur { .. } => blur_config(id, rows, on_edit),
        Operation::Posterize { .. } => posterize_config(id, rows, on_edit),
        Operation::Normalize { .. } => normalize_config(id, rows, on_edit),
        Operation::Downsample { .. } => downsample_config(id, rows, on_edit),
        Operation::Upscale { .. } => upscale_config(id, rows, on_edit),
        Operation::PaletteMap { colors, dither } => palette_map_config(id, rows, on_edit),
        _ => view! {
            <p class="text-xs text-slate-600 italic">"No editable parameters yet."</p>
        }
        .into_any(),
    }
}
