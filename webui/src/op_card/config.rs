mod color_picker;
mod dither;
mod generic_config;
mod palette_map;
mod swatches;

use generic_config::generic_op_config;
use palette_map::palette_map_config;

use crate::{EditPayload, OpRow};
use leptos::prelude::*;

/// Dispatch a config view by op tag. The five scalar ops go through the generic
/// descriptor-driven renderer; palette_map is the one special case (its params
/// aren't scalars). No serde helpers live here anymore — the store is a value
/// bag the widgets read and write directly.
pub fn op_config_view(
    id: usize,
    tag: &str,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match tag {
        "palette_map" => palette_map_config(id, rows, on_edit),
        _ => generic_op_config(id, tag, rows, on_edit),
    }
}
