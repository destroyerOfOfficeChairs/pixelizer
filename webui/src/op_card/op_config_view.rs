use crate::OpRow;
use leptos::prelude::*;
use pixelizer_core::Operation;
type EditPayload = (usize, Box<dyn Fn(&mut Operation)>);

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

// Returns the per-variant config inputs. Only Blur and Posterize are
// editable for now; the rest show a placeholder. This is the pattern
// you'll replicate for every variant.
pub fn op_config_view(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match op {
        Operation::Blur { .. } => {
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

            // One place that commits a new value: quantize, then push to rows.
            let commit = move |raw: f32| {
                let v = round1(raw); // quantize on store → JSON shows 5.7, not 5.73
                on_edit.run((
                    id,
                    Box::new(move |op| {
                        if let Operation::Blur { sigma } = op {
                            *sigma = v;
                        }
                    }),
                ));
            };

            view! {
                <label class="text-xs text-slate-400 block">
                    "sigma: "
                    <input
                        type="number" min="0" max="10" step="0.1"
                        prop:value=display
                        // TODO class=???
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
        Operation::Posterize { .. } => {
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
                on_edit.run((
                    id,
                    Box::new(move |op| {
                        if let Operation::Posterize { levels } = op {
                            *levels = raw;
                        }
                    }),
                ));
            };
            view! {
                <label class="text-xs text-slate-400 block">
                    "levels: "
                    <input
                        type="number" min="2" max="16" step="1"
                        prop:value=levels
                        // TODO class=???
                        on:change=move |ev| {
                            let v: u32 = event_target_value(&ev).parse().unwrap_or(2);
                            commit(v)
                        }
                    />
                    <input
                        type="range" min="2" max="16" step="1"
                        prop:value=levels
                        class="w-full accent-teal-500"
                        on:input=move |ev| {
                            let v: u32 = event_target_value(&ev).parse().unwrap_or(2);
                            commit(v)
                        }
                    />
                </label>
            }
            .into_any()
        }
        _ => view! {
            <p class="text-xs text-slate-600 italic">"No editable parameters yet."</p>
        }
        .into_any(),
    }
}
