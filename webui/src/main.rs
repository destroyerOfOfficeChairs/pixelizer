use leptos::prelude::*;
use pixelizer_core::{Operation, Pipeline, TrimMode};

#[derive(Clone)]
struct OpRow {
    id: usize,
    op: Operation,
}

fn op_label(op: &Operation) -> &'static str {
    match op {
        Operation::PixelSize { .. } => "Pixel Size",
        Operation::TrimHeight { .. } => "Trim Height",
        Operation::TrimWidth { .. } => "Trim Width",
        Operation::Downsample => "Downsample",
        Operation::PaletteMap { .. } => "Palette Map",
        Operation::Upscale { .. } => "Upscale",
        Operation::Posterize { .. } => "Posterize",
        Operation::Blur { .. } => "Blur",
        Operation::Normalize { .. } => "Normalize",
    }
}

fn default_op(label: &str) -> Operation {
    match label {
        "Pixel Size" => Operation::PixelSize { size: 8 },
        "Trim Height" => Operation::TrimHeight { mode: TrimMode::Both },
        "Trim Width" => Operation::TrimWidth { mode: TrimMode::Both },
        "Downsample" => Operation::Downsample,
        "Palette Map" => Operation::PaletteMap { colors: vec![], dither: None },
        "Upscale" => Operation::Upscale { factor: 4 },
        "Posterize" => Operation::Posterize { levels: 4 },
        "Blur" => Operation::Blur { sigma: 1.0 },
        "Normalize" => Operation::Normalize { low: 0.01, high: 0.99 },
        _ => Operation::Downsample,
    }
}

const ALL_LABELS: &[&str] = &[
    "Pixel Size", "Trim Height", "Trim Width", "Downsample",
    "Palette Map", "Upscale", "Posterize", "Blur", "Normalize",
];

#[component]
fn PipelineList() -> impl IntoView {
    let next_id = StoredValue::new(2);
    let (rows, set_rows) = signal(vec![
        OpRow { id: 0, op: Operation::Blur { sigma: 4.0 } },
        OpRow { id: 1, op: Operation::Posterize { levels: 5 } },
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
        set_rows.update(|rows| rows.push(OpRow { id, op: default_op(&label) }));
    };

    // Update one row's op in place by id, via a closure that mutates the Operation.
    let edit_op = move |id: usize, f: Box<dyn Fn(&mut Operation)>| {
        set_rows.update(|rows| {
            if let Some(r) = rows.iter_mut().find(|r| r.id == id) {
                f(&mut r.op);
            }
        });
    };

    let pipeline_json = move || {
        let ops: Vec<Operation> = rows.get().into_iter().map(|r| r.op).collect();
        let pipeline = Pipeline { operations: ops };
        serde_json::to_string_pretty(&pipeline).unwrap_or_else(|e| format!("error: {e}"))
    };

    view! {
        <div class="max-w-md mx-auto p-4 flex flex-col gap-3">
            <h3 class="text-lg font-bold text-teal-300">"Pipeline"</h3>
            <div class="flex flex-col gap-3">
                <For
                    each=move || rows.get()
                    key=|r| r.id
                    children=move |r| {
                        let id = r.id;
                        let label = op_label(&r.op);
                        view! {
                            <div class="rounded-lg border border-slate-800 bg-slate-800/30 overflow-hidden">
                                <div class="flex items-center gap-2 px-3 py-2 bg-slate-800/50 border-b border-slate-800">
                                    <span class="font-bold text-teal-300 text-sm">{label}</span>
                                    <button
                                        class="ml-auto text-slate-500 hover:text-teal-300 px-1"
                                        on:click=move |_| move_op(id, -1)
                                    >"↑"</button>
                                    <button
                                        class="text-slate-500 hover:text-teal-300 px-1"
                                        on:click=move |_| move_op(id, 1)
                                    >"↓"</button>
                                    <button
                                        class="text-slate-500 hover:text-red-400 text-lg leading-none px-1"
                                        on:click=move |_| remove_op(id)
                                    >"×"</button>
                                </div>
                                <div class="p-3">
                                    {op_config_view(id, &r.op, edit_op)}
                                </div>
                            </div>
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

// Returns the per-variant config inputs. Only Blur and Posterize are
// editable for now; the rest show a placeholder. This is the pattern
// you'll replicate for every variant.
fn op_config_view(
    id: usize,
    op: &Operation,
    edit_op: impl Fn(usize, Box<dyn Fn(&mut Operation)>) + Copy + 'static,
) -> AnyView {
    match op {
        Operation::Blur { sigma } => {
            let current = *sigma;
            view! {
                <label class="text-xs text-slate-400 block">
                    "sigma: " {move || format!("{current:.1}")}
                    <input
                        type="range" min="0" max="10" step="0.1"
                        prop:value=current
                        class="w-full accent-teal-500"
                        on:input=move |ev| {
                            let v: f32 = event_target_value(&ev).parse().unwrap_or(0.0);
                            edit_op(id, Box::new(move |op| {
                                if let Operation::Blur { sigma } = op { *sigma = v; }
                            }));
                        }
                    />
                </label>
            }.into_any()
        }
        Operation::Posterize { levels } => {
            let current = *levels;
            view! {
                <label class="text-xs text-slate-400 block">
                    "levels: " {move || current.to_string()}
                    <input
                        type="range" min="2" max="16" step="1"
                        prop:value=current
                        class="w-full accent-teal-500"
                        on:input=move |ev| {
                            let v: u32 = event_target_value(&ev).parse().unwrap_or(2);
                            edit_op(id, Box::new(move |op| {
                                if let Operation::Posterize { levels } = op { *levels = v; }
                            }));
                        }
                    />
                </label>
            }.into_any()
        }
        _ => view! {
            <p class="text-xs text-slate-600 italic">"No editable parameters yet."</p>
        }.into_any(),
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(PipelineList);
}
