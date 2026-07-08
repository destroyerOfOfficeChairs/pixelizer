use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;

#[component]
pub fn Swatches(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    palette_key: String,
) -> AnyView {
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
                children=move |(sid, hex)| view! {
                    <div
                        class="group relative w-8 h-8 cursor-pointer"
                        style:background-color=hex
                        on:click=move |ev: leptos::ev::MouseEvent| swatch_clicked(ev)
                    >
                        <span class="absolute -top-1 -right-1 hidden group-hover:flex
                                    items-center justify-center w-4 h-4 rounded-full
                                    bg-slate-900 text-slate-200 text-xs leading-none
                                    border border-slate-600"
                            on:click={
                                let value = palette_key.clone();
                                move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                owned.update(|v| v.retain(|(swatch_id, _)| *swatch_id != sid));
                                let colors: Vec<String> = owned.with(|v| v.iter().map(|(_, h)| h.clone()).collect());
                                on_edit.run((id, value.clone(), ParamValue::Palette(colors)));
                            }
                            }>
                            "×"
                        </span>
                    </div>
                }
            />
        </div>
    }
    .into_any()
}

fn swatch_clicked(ev: leptos::ev::MouseEvent) {
    ev.stop_propagation();
    let target = event_target::<web_sys::Element>(&ev);
    let rect = target.get_bounding_client_rect();
    leptos::logging::log!("clicked a swatch");
    leptos::logging::log!("x: {}", rect.x());
    leptos::logging::log!("y: {}", rect.y());
    leptos::logging::log!("width: {}", rect.width());
    leptos::logging::log!("height: {}", rect.height());
    let x = rect.x();
    let y = rect.y() + rect.height();
    spawn_picker(x, y);
}

fn spawn_picker(x: f64, y: f64) {
    leptos::logging::log!("Hello from spawn_picker()");
}
