use super::color_picker::ColorPicker;
use crate::op_card::config::palette_map::MAX_PALETTE_COLORS;
use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow};
use leptos::portal::Portal;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct PickerAnchor {
    pub sid: usize,
    pub x: f64,
    pub y: f64,
    pub is_new: bool,
}

/// Picker height, measured once in devtools.
const PICKER_HEIGHT: f64 = 320.0;
/// Gap between swatch and picker, matching the natural spacing below.
const PICKER_GAP: f64 = 4.0;

#[component]
pub fn Swatches(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    palette_key: &'static str,
) -> AnyView {
    let owned = RwSignal::new(Vec::<(usize, String)>::new());
    let next_id = StoredValue::new(0usize);
    let editing: RwSignal<Option<PickerAnchor>> = RwSignal::new(None);
    let at_cap = Signal::derive(move || owned.with(|v| v.len() >= MAX_PALETTE_COLORS));

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
        <div class="flex flex-wrap gap-1 px-3 py-2 max-h-64 overflow-y-auto">
            <For
                each=move || owned.get()
                key=|(id, _)| *id
                children=move |(sid, _)| {
                    let hex = Signal::derive(move || {
                        owned.with(|v| v.iter()
                            .find(|(s, _)| *s == sid)
                            .map(|(_, h)| h.clone())
                            .unwrap_or_default())
                    });
                    view! {
                        <div
                            style:background-color=move || hex.get()
                            class="group relative w-8 h-8 cursor-pointer"
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
                }
            />
            <div
                class="w-8 h-8 border border-dashed border-slate-600 rounded
                    flex items-center justify-center text-slate-500
                    hover:border-slate-400 hover:text-slate-300 cursor-pointer"
                class:hidden=move || at_cap.get()
                on:click=move |ev: leptos::ev::MouseEvent| {
                    let n = next_id.get_value();
                    next_id.set_value(n + 1);
                    owned.update(|v| v.push((n, "#808080".to_string())));
                    let colors: Vec<String> = owned.with(|v| v.iter().map(|(_, h)| h.clone()).collect());
                    on_edit.run((id, palette_key.to_string(), ParamValue::Palette(colors)));

                    // let target = event_target::<web_sys::Element>(&ev);
                    // let rect = target.get_bounding_client_rect();
                    // editing.set(Some(PickerAnchor {
                    //     sid: n,
                    //     x: rect.x(),
                    //     y: rect.y() + rect.height(),
                    //     is_new: true,
                    // }));
                    let target = event_target::<web_sys::Element>(&ev);
                    let rect = target.get_bounding_client_rect();
                    let (x, y) = anchor_for(&rect);
                    editing.set(Some(PickerAnchor { sid: n, x, y, is_new: true }));
                }
            >
                "+"
            </div>
            {move || editing.get().map(|anchor| {
                view! {
                    <Portal>
                        <ColorPicker
                            anchor=anchor
                            hex=picker_hex
                            on_close=Callback::new(move |_| {
                                // If this was a new (pre-created) swatch, cancel means "undo the create":
                                // remove the swatch and re-commit the palette without it.
                                if anchor.is_new {
                                    owned.update(|v| v.retain(|(s, _)| *s != anchor.sid));
                                    let colors: Vec<String> = owned.with(|v| v.iter().map(|(_, h)| h.clone()).collect());
                                    on_edit.run((id, palette_key.to_string(), ParamValue::Palette(colors)));
                                }
                                editing.set(None);
                            })
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
    let (x, y) = anchor_for(&rect);
    editing.set(Some(PickerAnchor {
        sid,
        x,
        y,
        is_new: false,
    }));
}

/// Where to put the picker for a swatch at `rect`: below if it fits,
/// flipped above if it would run off the bottom of the viewport.
fn anchor_for(rect: &web_sys::DomRect) -> (f64, f64) {
    let viewport_h = window()
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);

    let below = rect.y() + rect.height() + PICKER_GAP;
    let y = if below + PICKER_HEIGHT > viewport_h {
        rect.y() - PICKER_HEIGHT - PICKER_GAP // flip above
    } else {
        below
    };
    (rect.x(), y)
}
