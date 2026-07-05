use crate::OpRow;
use crate::op_instance::ParamValue;
use leptos::prelude::*;

#[component]
pub fn Swatches(id: usize, rows: ReadSignal<Vec<OpRow>>, palette_key: String) -> AnyView {
    // The current palette colors, read from the bag. (Swatch display is future
    // work; kept live so it reflects the selected palette when built out.)
    let colors = Signal::derive(move || {
        rows.with(|rs| {
            rs.iter()
                .find(|r| r.id == id)
                .and_then(|r| r.inst.values.get(&palette_key))
                .and_then(|v| match v {
                    ParamValue::Palette(c) => Some(c.clone()),
                    _ => None,
                })
        })
        .unwrap_or_else(|| vec!["#000000".to_owned(), "#ffffff".to_owned()])
    });

    view! {
        <div class="flex flex-wrap gap-1 px-3 py-2">
            <For
                each=move || colors.get()
                key=|hex| hex.clone()
                children=move |hex| {
                    view! {
                        <div class="w-8 h-8" style:background-color=hex></div>
                    }
                }
            />
        </div>
    }
    .into_any()
}
