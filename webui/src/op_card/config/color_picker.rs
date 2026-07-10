use super::swatches::PickerAnchor;
use leptos::prelude::*;

#[component]
pub fn ColorPicker(anchor: PickerAnchor, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="fixed z-50 w-48 h-48 bg-slate-800 border border-slate-600 rounded shadow-lg"
            style:left=format!("{}px", anchor.x)
            style:top=format!("{}px", anchor.y)
        >
        <button on:click=move |_| on_close.run(())>"×"</button>
            "picker for swatch " {anchor.sid}
        </div>
    }
}
