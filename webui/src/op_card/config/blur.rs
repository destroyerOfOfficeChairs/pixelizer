use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

pub fn blur_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let sigma = move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Blur { sigma } => Some(sigma),
                _ => None,
            })
            .unwrap_or(0.0)
    };
    let display = move || format!("{:.1}", sigma());

    let commit = move |raw: f32| {
        let v = round1(raw);
        on_edit.run((id, Box::new(move |op| {
            if let Operation::Blur { sigma } = op { *sigma = v; }
        })));
    };

    view! {
        <label class="text-xs text-slate-400 block">
            "sigma: "
            <input
                type="number" min="0" max="10" step="0.1"
                prop:value=display
                on:change=move |ev| {
                    let raw: f32 = event_target_value(&ev).parse().unwrap_or(0.0);
                    commit(raw);
                }
            />
            <input
                type="range" min="0" max="10" step="0.1"
                prop:value=display
                class="w-full accent-teal-500"
                on:input=move |ev| {
                    let raw: f32 = event_target_value(&ev).parse().unwrap_or(0.0);
                    commit(raw);
                }
            />
        </label>
    }
    .into_any()
}
