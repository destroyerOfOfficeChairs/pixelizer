mod generic_config;
mod palette_map;

use generic_config::generic_op_config;
use palette_map::palette_map_config;

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
        Operation::Posterize { .. }
        | Operation::Normalize { .. }
        | Operation::Downsample { .. }
        | Operation::Upscale { .. }
        | Operation::Blur { .. } => generic_op_config(id, op, rows, on_edit),

        // --- special-case ---
        Operation::PaletteMap { .. } => palette_map_config(id, rows, on_edit),
    }
}
