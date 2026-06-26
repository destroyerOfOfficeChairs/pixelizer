mod op_card;
use leptos::prelude::*;
use op_card::OpCard;
use pixelizer_core::{Operation, Pipeline, TrimMode};
type EditPayload = (usize, Box<dyn Fn(&mut Operation)>);

#[derive(Clone)]
struct OpRow {
    id: usize,
    op: Operation,
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

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(PipelineList);
}
