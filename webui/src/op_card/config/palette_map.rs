use super::dither::DitherConfig;
use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow, Palettes};
use leptos::prelude::*;

const PALETTE_KEY: &str = "palette";

pub fn palette_map_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    // The current palette colors, read from the bag. (Swatch display is future
    // work; kept live so it reflects the selected palette when built out.)
    let _colors = Signal::derive(move || {
        rows.with(|rs| {
            rs.iter()
                .find(|r| r.id == id)
                .and_then(|r| r.inst.values.get(PALETTE_KEY))
                .and_then(|v| match v {
                    ParamValue::Palette(c) => Some(c.clone()),
                    _ => None,
                })
        })
        .unwrap_or_else(|| vec!["#000000".to_owned(), "#ffffff".to_owned()])
    });

    let preloaded_palettes =
        use_context::<StoredValue<Palettes>>().expect("You forgot to provide palettes.");

    let options = preloaded_palettes.with_value(|p| {
        p.palettes
            .iter()
            .map(|(name, _colors)| {
                view! { <option value=name.clone()>{name.clone()}</option> }
            })
            .collect_view()
    });

    let on_change = move |ev| {
        let chosen = event_target_value(&ev);
        let colors = preloaded_palettes.with_value(|p| {
            p.palettes
                .iter()
                .find(|(name, _)| *name == chosen)
                .map(|(_, colors)| colors.clone())
        });
        if let Some(colors) = colors {
            on_edit.run((id, PALETTE_KEY.to_string(), ParamValue::Palette(colors)));
        }
    };

    view! {
        <div class="flex flex-col">
            <div class="px-3 pt-1">
                <select
                    class="bg-slate-900 border border-slate-700 rounded-md text-sm \
                           text-slate-200 p-2 w-full"
                    on:change=on_change
                >
                    <option value="">"— pick a palette —"</option>
                    {options}
                </select>
            </div>

            <DitherConfig id=id rows=rows on_edit=on_edit/>
        </div>
    }
    .into_any()
}
