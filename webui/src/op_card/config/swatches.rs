use crate::OpRow;
use crate::op_instance::ParamValue;
use leptos::prelude::*;

#[component]
pub fn Swatches(id: usize, rows: ReadSignal<Vec<OpRow>>, palette_key: String) -> AnyView {
    // Owned, stable id'd list. NOT derived — identity has to persist between reads.
    let owned = RwSignal::new(Vec::<(usize, String)>::new());
    let next_id = StoredValue::new(0usize);

    // Read the bag's plain Vec<String> for this op. THIS can be derived — it's pure.
    let bag_colors = {
        let palette_key = palette_key.clone();
        Signal::derive(move || {
            rows.with(|rs| {
                rs.iter()
                    .find(|r| r.id == id) // `id` here is the prop, un-shadowed
                    .and_then(|r| r.inst.values.get(&palette_key))
                    .and_then(|v| match v {
                        ParamValue::Palette(c) => Some(c.clone()),
                        _ => None,
                    })
            })
            .unwrap_or_else(|| vec!["#000000".into(), "#ffffff".into()])
        })
    };

    // The sync effect: reconcile owned <- bag when the bag changes from OUTSIDE.
    Effect::new(move |_| {
        let incoming = bag_colors.get();
        let is_echo = owned.with_untracked(|o| {
            o.len() == incoming.len() && o.iter().zip(&incoming).all(|((_, h), inc)| h == inc)
        });
        if !is_echo {
            leptos::logging::log!("re-mint: {} colors", incoming.len());
            // Outside change (dropdown, import): re-mint fresh ids.
            let minted = incoming
                .into_iter()
                .map(|h| {
                    let n = next_id.get_value();
                    next_id.set_value(n + 1);
                    (n, h)
                })
                .collect();
            owned.set(minted);
        }
    });

    view! {
        <div class="flex flex-wrap gap-1 px-3 py-2">
            <For
                each=move || owned.get()
                key=|(id, _)| *id
                children=move |(_, hex)| view! {
                    <div class="w-8 h-8" style:background-color=hex></div>
                }
            />
        </div>
    }
    .into_any()
}
