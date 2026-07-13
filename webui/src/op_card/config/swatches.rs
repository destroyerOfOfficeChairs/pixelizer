use super::color_picker::ColorPicker;
use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow};
use leptos::portal::Portal;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct PickerAnchor {
    pub sid: usize,
    pub x: f64,
    pub y: f64,
}

#[component]
pub fn Swatches(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    palette_key: &'static str,
) -> AnyView {
    // 1. Revert to plain Strings
    let owned = RwSignal::new(Vec::<(usize, String)>::new());
    let next_id = StoredValue::new(0usize);
    let editing: RwSignal<Option<PickerAnchor>> = RwSignal::new(None);

    let bag_colors = {
        Signal::derive(move || {
            rows.with(|rs| {
                rs.iter()
                    .find(|r| r.id == id)
                    .and_then(|r| r.inst.values.get(palette_key))
                    .and_then(|v| match v {
                        ParamValue::Palette(c) => Some(c.clone()),
                        _ => None,
                    })
            })
            .unwrap_or_else(|| vec!["#000000".into(), "#ffffff".into()])
        })
    };

    let picker_hex = Signal::derive(move || {
        editing
            .get()
            .and_then(|a| {
                owned.with(|v| {
                    v.iter()
                        .find(|(sid, _)| *sid == a.sid)
                        .map(|(_, h)| h.clone())
                })
            })
            .unwrap_or_else(|| "#000000".to_string())
    });

    Effect::new(move |_| {
        let incoming = bag_colors.get();
        let is_echo = owned.with_untracked(|o| {
            // Safe plain string comparison without disposed signals
            o.len() == incoming.len() && o.iter().zip(&incoming).all(|((_, h), inc)| h == inc)
        });
        if !is_echo {
            leptos::logging::log!("re-mint: {} colors", incoming.len());
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
                // 2. The Magic Fix: Leptos re-renders the item if the ID or String changes
                key=|(id, hex)| (*id, hex.clone())
                children=move |(sid, hex)| view! {
                    <div
                        class="group relative w-8 h-8 cursor-pointer"
                        style:background-color=hex.clone()
                        on:click=move |ev: leptos::ev::MouseEvent| swatch_clicked(ev, editing, sid)
                    >
                        <span class="absolute -top-1 -right-1 hidden group-hover:flex
                                    items-center justify-center w-4 h-4 rounded-full
                                    bg-slate-900 text-slate-200 text-xs leading-none
                                    border border-slate-600"
                            on:click={
                                move |ev: leptos::ev::MouseEvent| {
                                    ev.stop_propagation();
                                    owned.update(|v| v.retain(|(swatch_id, _)| *swatch_id != sid));
                                    let colors: Vec<String> = owned.with(|v| v.iter().map(|(_, h)| h.clone()).collect());
                                    on_edit.run((id, palette_key.to_string(), ParamValue::Palette(colors)));
                                }
                            }>
                            "×"
                        </span>
                    </div>
                }
            />
            {move || editing.get().map(|anchor| {
                view! {
                    <Portal>
                        <ColorPicker
                            anchor=anchor
                            hex=picker_hex
                            on_close=Callback::new(move |_| editing.set(None))
                            on_pick=Callback::new(move |new_hex: String| {
                                let sid = anchor.sid;
                                owned.update(|v| {
                                    if let Some((_, h)) = v.iter_mut().find(|(s, _)| *s == sid) {
                                        *h = new_hex; // Update the plain string
                                    }
                                });
                                let colors: Vec<String> = owned.with(|v| v.iter().map(|(_, h)| h.clone()).collect());
                                on_edit.run((id, palette_key.to_string(), ParamValue::Palette(colors)));
                                editing.set(None);   // Safely closes after commit
                            })
                        />
                    </Portal>
                }
            })}
        </div>
    }
    .into_any()
}

fn swatch_clicked(ev: leptos::ev::MouseEvent, editing: RwSignal<Option<PickerAnchor>>, sid: usize) {
    ev.stop_propagation();
    let target = event_target::<web_sys::Element>(&ev);
    let rect = target.get_bounding_client_rect();
    let x = rect.x();
    let y = rect.y() + rect.height();
    leptos::logging::log!("clicked a swatch");
    leptos::logging::log!("x: {}", rect.x());
    leptos::logging::log!("y: {}", rect.y());
    leptos::logging::log!("width: {}", rect.width());
    leptos::logging::log!("height: {}", rect.height());
    editing.set(Some(PickerAnchor {
        sid: sid,
        x: x,
        y: y,
    }));
}
