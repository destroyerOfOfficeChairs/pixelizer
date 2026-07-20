use crate::Palettes;
use leptos::{html, portal::Portal, prelude::*};
use leptos_use::{on_click_outside, use_event_listener};
use web_sys::wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
struct DropdownAnchor {
    x: f64,
    y: f64,
    w: f64,
}

#[component]
pub fn PaletteDropdown(
    palettes: RwSignal<Palettes>,
    on_select: Callback<Vec<String>>,
) -> impl IntoView {
    let dropdown_ref = NodeRef::<leptos::html::Div>::new();
    let button_ref = NodeRef::<html::Button>::new();
    let max_height = "800px";
    let open: RwSignal<Option<DropdownAnchor>> = RwSignal::new(None);
    let on_click = move |ev: leptos::ev::MouseEvent| {
        let target = event_target::<web_sys::Element>(&ev);
        let rect = target.get_bounding_client_rect();
        let dims = DropdownAnchor {
            x: rect.x(),
            y: rect.y() + rect.height(),
            w: rect.width(),
        };
        if open.get_untracked().is_some() {
            open.set(None);
        } else {
            open.set(Some(dims));
        }
    };
    let _ = use_event_listener(window(), leptos::ev::scroll, move |_| open.set(None));
    let _ = on_click_outside(dropdown_ref, move |ev| {
        if let Some(btn) = button_ref.get_untracked() {
            if let Some(target) = ev.target() {
                if let Ok(node) = target.dyn_into::<web_sys::Node>() {
                    if btn.contains(Some(&node)) {
                        return;
                    }
                }
            }
        }
        open.set(None);
    });
    view! {
        <button
            class="bg-teal-600 hover:bg-teal-500 text-white font-bold rounded-md px-4 py-2"
            on:click=on_click
            node_ref=button_ref
        >
            "Select Palette"
        </button>
        {move || open.get().map(|anchor| view! {
            <Portal>
                <div
                    class="fixed z-50 bg-slate-800 border border-slate-700 rounded-md \
                        shadow-xl overflow-y-auto py-1"
                    style:left=format!("{}px", anchor.x)
                    style:top=format!("{}px", anchor.y)
                    style:width=format!("{}px", anchor.w)
                    style:max-height=max_height
                    node_ref=dropdown_ref
                >
                    // rows here
                    {move || palettes.with(|p| {
                        p.palettes.iter().map(|(name, colors)| {
                            let colors = colors.clone();
                            view! {
                                <div
                                    class="flex items-center gap-2 px-3 py-1.5 cursor-pointer hover:bg-slate-700"
                                    on:click=move |_| {
                                        on_select.run(colors.clone());
                                        open.set(None);
                                    }
                                >
                                    <span class="text-sm text-slate-200 flex-1 truncate">{name.clone()}</span>
                                    // swatch strip goes here
                                </div>
                            }
                        }).collect_view()
                    })}
                </div>
            </Portal>
        })}
    }
}
