use crate::OpRow;
use crate::op_instance::default_instance;
use leptos::prelude::*;
use pixelizer_core::op_schema::all_op_menu;

#[component]
pub fn AddOp(set_rows: WriteSignal<Vec<OpRow>>) -> impl IntoView {
    let next_id = StoredValue::new(1);
    let add_op = move |tag: String| {
        let Some(inst) = default_instance(&tag) else {
            return;
        };
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        set_rows.update(|rows| rows.push(OpRow { id, inst }));
    };
    view! {
        <select
            class="bg-slate-900 border border-slate-700 rounded-md text-sm text-slate-200 p-2"
            on:change=move |ev| {
                let tag = event_target_value(&ev);
                if !tag.is_empty() { add_op(tag); }
            }
        >
            <option value="">"+ Add operation…"</option>
            {all_op_menu().into_iter().map(|(tag, label)| view! {
                <option value=tag>{label}</option>
            }).collect_view()}
        </select>
    }
}
