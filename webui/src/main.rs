use leptos::html;
use leptos::prelude::*;
use pixelizer_core::{Operation, Pipeline, TrimMode};
type EditPayload = (usize, Box<dyn Fn(&mut Operation)>);

#[derive(Clone)]
struct OpRow {
    id: usize,
    op: Operation,
}

fn op_label(op: &Operation) -> &'static str {
    match op {
        Operation::Downsample { .. } => "Downsample",
        Operation::PaletteMap { .. } => "Palette Map",
        Operation::Upscale { .. } => "Upscale",
        Operation::Posterize { .. } => "Posterize",
        Operation::Blur { .. } => "Blur",
        Operation::Normalize { .. } => "Normalize",
    }
}

const ALL_LABELS: &[&str] = &[
    "Downsample",
    "Palette Map",
    "Upscale",
    "Posterize",
    "Blur",
    "Normalize",
];

fn default_op(label: &str) -> Operation {
    match label {
        "Downsample" => Operation::Downsample {
            pixel_size: 8,
            trim: TrimMode::TrimAll,
        },
        "Palette Map" => Operation::PaletteMap {
            colors: vec![],
            dither: None,
        },
        "Upscale" => Operation::Upscale { factor: 4 },
        "Posterize" => Operation::Posterize { levels: 4 },
        "Blur" => Operation::Blur { sigma: 1.0 },
        "Normalize" => Operation::Normalize {
            low: 0.01,
            high: 0.99,
        },
        _ => Operation::Downsample {
            pixel_size: 8,
            trim: TrimMode::TrimAll,
        },
    }
}

// ---- OpCard: one card. Top bar + collapsible animated settings area. ----
#[component]
fn OpCard(
    id: usize,
    op: Operation,
    rows: ReadSignal<Vec<OpRow>>,
    // Callbacks back to the parent — this is the chapter 3.9 part.
    on_move: Callback<i32>,
    on_remove: Callback<()>,
    on_edit: Callback<EditPayload>,
) -> impl IntoView {
    // Local UI state: is the settings area open? Lives in the card, because
    // it's nobody else's business — the parent doesn't care if a card is expanded.
    let (open, set_open) = signal(true);

    // A handle to the settings-content div, so we can measure its height.
    let content_ref: NodeRef<html::Div> = NodeRef::new();

    let label = op_label(&op);

    // The collapse animation. We drive the wrapper's max-height from a signal.
    // When open: measure the content's scrollHeight and use that. When closed: 0.
    let max_height = move || {
        if open.get() {
            // Read the real DOM node's scroll height, in px.
            content_ref
                .get()
                .map(|el| format!("{}px", el.scroll_height()))
                .unwrap_or_else(|| "1000px".to_string()) // fallback before first measure
        } else {
            "0px".to_string()
        }
    };

    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-800/30 overflow-hidden">
            // ---- Top bar (will become the drag handle in M2) ----
            <div class="flex items-center gap-2 px-3 py-2 bg-slate-800/50 border-b border-slate-800 cursor-grab select-none">
                // Show/hide toggle — NOT part of the future drag handle.
                <button
                    class="text-slate-500 hover:text-teal-300 px-1"
                    on:click=move |ev| {
                        ev.stop_propagation(); // so a future drag handler on the bar doesn't also fire
                        set_open.update(|o| *o = !*o);
                    }
                >
                    {move || if open.get() { "▾" } else { "▸" }}
                </button>
                <span class="font-bold text-teal-300 text-sm">{label}</span>
                <button
                    class="ml-auto text-slate-500 hover:text-teal-300 px-1"
                    on:click=move |_| on_move.run(-1)
                >"↑"</button>
                <button
                    class="text-slate-500 hover:text-teal-300 px-1"
                    on:click=move |_| on_move.run(1)
                >"↓"</button>
                <button
                    class="text-slate-500 hover:text-red-400 text-lg leading-none px-1"
                    on:click=move |_| on_remove.run(())
                >"×"</button>
            </div>

            // ---- Collapsible settings area ----
            // Outer wrapper animates max-height. overflow-hidden clips during collapse.
            <div
                class="overflow-hidden transition-[max-height] duration-200 ease-in-out"
                style:max-height=max_height
            >
                // Inner content is what we measure. Its natural height is the target.
                <div node_ref=content_ref class="p-3">
                    {op_config_view(id, &op, rows, on_edit)}
                </div>
            </div>
        </div>
    }
}

#[component]
fn PipelineList() -> impl IntoView {
    let next_id = StoredValue::new(2);
    let (rows, set_rows) = signal(vec![
        OpRow {
            id: 0,
            op: Operation::Blur { sigma: 4.0 },
        },
        OpRow {
            id: 1,
            op: Operation::Posterize { levels: 5 },
        },
    ]);

    let move_op = move |id: usize, dir: i32| {
        set_rows.update(|rows| {
            if let Some(i) = rows.iter().position(|r| r.id == id) {
                let j = i as i32 + dir;
                if j >= 0 && (j as usize) < rows.len() {
                    rows.swap(i, j as usize);
                }
            }
        });
    };
    let remove_op = move |id: usize| {
        set_rows.update(|rows| rows.retain(|r| r.id != id));
    };
    let add_op = move |label: String| {
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        set_rows.update(|rows| {
            rows.push(OpRow {
                id,
                op: default_op(&label),
            })
        });
    };

    let edit_op = Callback::new(move |(id, f): EditPayload| {
        set_rows.update(|rows| {
            if let Some(r) = rows.iter_mut().find(|r| r.id == id) {
                f(&mut r.op);
            }
        });
    });

    let pipeline_json = move || {
        let ops: Vec<Operation> = rows.get().into_iter().map(|r| r.op).collect();
        let pipeline = Pipeline { operations: ops };
        serde_json::to_string_pretty(&pipeline).unwrap_or_else(|e| format!("error: {e}"))
    };

    view! {
        <div class="max-w-md mx-auto p-4 flex flex-col gap-3">
            <h3 class="text-lg font-bold text-teal-300">"Pipeline"</h3>
            <div class="flex flex-col gap-3">
            // Inside PipelineList, the <For/> becomes:
            <For
                each=move || rows.get()
                key=|r| r.id
                children=move |r| {
                    let id = r.id;
                    view! {
                        <OpCard
                            id=id
                            op=r.op.clone()
                            rows=rows
                            on_move=Callback::new(move |dir: i32| move_op(id, dir))
                            on_remove=Callback::new(move |_| remove_op(id))
                            on_edit=edit_op
                        />
                    }
                }
            />
            </div>
            <select
                class="bg-slate-900 border border-slate-700 rounded-md text-sm text-slate-200 p-2"
                on:change=move |ev| {
                    let label = event_target_value(&ev);
                    if !label.is_empty() { add_op(label); }
                }
            >
                <option value="">"+ Add operation…"</option>
                {ALL_LABELS.iter().map(|l| view! {
                    <option value=*l>{*l}</option>
                }).collect_view()}
            </select>
            <h4 class="text-sm font-bold text-teal-300 mt-2">"Pipeline JSON"</h4>
            <pre class="text-xs bg-slate-950 text-slate-300 p-3 rounded overflow-x-auto">{pipeline_json}</pre>
        </div>
    }
}

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

// Returns the per-variant config inputs. Only Blur and Posterize are
// editable for now; the rest show a placeholder. This is the pattern
// you'll replicate for every variant.
fn op_config_view(
    id: usize,
    op: &Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    match op {
        Operation::Blur { .. } => {
            // Live read of THIS op's sigma from the single source of truth.
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

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(PipelineList);
}
