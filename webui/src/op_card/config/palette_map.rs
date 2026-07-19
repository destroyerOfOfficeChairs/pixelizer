mod color_picker;
mod dither;
mod dropzone;
mod swatches;

use super::generic_config::BoolWidget;
use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow, Palettes};
use dither::DitherConfig;
use dropzone::PaletteDropZone;
use leptos::prelude::*;
use swatches::Swatches;

const PALETTE_KEY: &'static str = "palette";

pub fn palette_map_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let preloaded_palettes =
        use_context::<RwSignal<Palettes>>().expect("You forgot to provide palettes.");

    let options = move || {
        preloaded_palettes.with(|p| {
            p.palettes
                .iter()
                .map(|(name, _colors)| {
                    view! { <option value=name.clone()>{name.clone()}</option> }
                })
                .collect_view()
        })
    };

    let on_change = move |ev| {
        let chosen = event_target_value(&ev);
        let colors = preloaded_palettes.with(|p| {
            p.palettes
                .iter()
                .find(|(name, _)| *name == chosen)
                .map(|(_, colors)| colors.clone())
        });
        if let Some(colors) = colors {
            on_edit.run((id, PALETTE_KEY.to_string(), ParamValue::Palette(colors)));
        }
    };

    let on_load = Callback::new(move |(name, colors): (String, Vec<String>)| {
        preloaded_palettes.update(|p| {
            // Re-uploading a same-named file replaces the old entry
            // instead of duplicating it.
            p.palettes.retain(|(n, _)| *n != name);
            p.palettes.push((name.clone(), colors.clone()));
            p.palettes.sort_by(|a, b| a.0.cmp(&b.0));
        });
        // Auto-select the upload: committing its colors is what "selecting
        // a palette" means everywhere else, so do the same here.
        on_edit.run((id, PALETTE_KEY.to_string(), ParamValue::Palette(colors)));
    });

    view! {
        <div class="flex flex-col">
            <PaletteDropZone on_load=on_load />
            <div class="px-3 pt-1">
                <select
                    class="bg-slate-900 border border-slate-700 rounded-md text-sm \
                           text-slate-200 p-2 w-full"
                    on:change=on_change
                >
                    // <option value="">"— pick a palette —"</option>
                    //
                    // "Black and White" option auto-magically appears ¯\_(ツ)_/¯
                    {options}
                </select>
            </div>

            <Swatches id=id rows=rows on_edit=on_edit palette_key=PALETTE_KEY/>

            // TODO: Remove hardcoded "default=true", "key=alpha", and "label=preserve alpha" in favor of reading from the op_schema
            <BoolWidget id=id rows=rows on_edit=on_edit default=true key="alpha" label="preserve alpha"/>

            <DitherConfig id=id rows=rows on_edit=on_edit/>
        </div>
    }
    .into_any()
}
