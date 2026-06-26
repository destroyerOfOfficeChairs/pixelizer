use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

pub fn posterize_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let levels = move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Posterize { levels } => Some(levels),
                _ => None,
            })
            .unwrap_or(0)
    };

    let commit = move |raw: u32| {
        on_edit.run((id, Box::new(move |op| {
            if let Operation::Posterize { levels } = op { *levels = raw; }
        })));
    };

    view! {
        <label class="text-xs text-slate-400 block">
            "levels: "
            <input
                type="number" min="2" max="16" step="1"
                prop:value=levels
                on:change=move |ev| {
                    let v: u32 = event_target_value(&ev).parse().unwrap_or(2);
                    commit(v);
                }
            />
            <input
                type="range" min="2" max="16" step="1"
                prop:value=levels
                class="w-full accent-teal-500"
                on:input=move |ev| {
                    let v: u32 = event_target_value(&ev).parse().unwrap_or(2);
                    commit(v);
                }
            />
        </label>
    }
    .into_any()
}
