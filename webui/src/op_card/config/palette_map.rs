use crate::{EditPayload, OpRow, Palettes};
use leptos::prelude::*;
use pixelizer_core::Operation;

pub fn palette_map_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    // prefix with an underscore for now.
    let _colors_to_map = Signal::derive(move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match &r.op {
                Operation::PaletteMap { colors, dither: _ } => Some(colors.clone()),
                _ => None,
            })
            .unwrap_or(vec!["#000000".to_owned(), "#ffffff".to_owned()])
    });

    let preloaded_palettes =
        use_context::<StoredValue<Palettes>>().expect("You forgot to provide palettes.");

    // Build the <option> list by reading through the handle.
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
        // Look up the chosen palette's colors, then push an edit.
        let colors = preloaded_palettes.with_value(|p| {
            p.palettes
                .iter()
                .find(|(name, _)| *name == chosen)
                .map(|(_, colors)| colors.clone())
        });
        if let Some(colors) = colors {
            on_edit.run((
                id,
                Box::new(move |op: &mut Operation| {
                    if let Operation::PaletteMap { colors: c, .. } = op {
                        *c = colors.clone();
                    }
                }),
            ));
        }
    };

    view! {
        <select
            class="bg-slate-900 border border-slate-700 rounded-md text-sm text-slate-200 p-2 w-full"
            on:change=on_change
        >
            <option value="">"— pick a palette —"</option>
            {options}
        </select>
    }
    .into_any()
}
