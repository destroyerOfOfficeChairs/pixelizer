use crate::op_instance::default_instance;
use crate::{EditPayload, OpRow, op_card};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use op_card::OpCard;
use pixelizer_core::Pipeline;
use pixelizer_core::op_schema::all_op_menu;

#[component]
pub fn PipelineList(
    rows: ReadSignal<Vec<OpRow>>,
    set_rows: WriteSignal<Vec<OpRow>>,
    on_run: Callback<()>,
    can_run: Signal<bool>,
) -> impl IntoView {
    let next_id = StoredValue::new(1);
    let show_debug = signal(false); // (read, write) tuple
    let (show_debug_read, show_debug_write) = show_debug;
    let copied = RwSignal::new(false);

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
    let add_op = move |tag: String| {
        let Some(inst) = default_instance(&tag) else {
            return;
        };
        let id = next_id.get_value();
        next_id.set_value(id + 1);
        set_rows.update(|rows| rows.push(OpRow { id, inst }));
    };

    // The single write path for every param edit. A Config emits (id, key,
    // value); we drop it into that instance's bag. No closures, no serde.
    let edit_op = Callback::new(move |(id, key, value): EditPayload| {
        set_rows.update(|rows| {
            if let Some(r) = rows.iter_mut().find(|r| r.id == id) {
                r.inst.values.insert(key, value);
            }
        });
    });

    // Serialize lazily. Only called when the preview is shown, and re-runs
    // reactively as `rows` changes while shown. Builds the typed pipeline via
    // the boundary conversion; a malformed bag renders as an error string.
    let pipeline_yaml = move || {
        let built: Result<Vec<_>, _> = rows
            .get()
            .into_iter()
            .map(|r| r.inst.to_operation())
            .collect();
        match built {
            Ok(operations) => {
                let pipeline = Pipeline { operations };
                serde_yaml::to_string(&pipeline).unwrap_or_else(|e| format!("error: {e}"))
            }
            Err(e) => format!("error: {e}"),
        }
    };

    // Copy to clipboard, then flash "Copied!" for ~1.2s. We only flip `copied`
    // to true after the clipboard write resolves, so the label reflects reality.
    let copy_yaml = move |_| {
        let text = pipeline_yaml();
        spawn_local(async move {
            let Some(window) = web_sys::window() else {
                return;
            };
            let clipboard = window.navigator().clipboard();
            let promise = clipboard.write_text(&text);
            if wasm_bindgen_futures::JsFuture::from(promise).await.is_ok() {
                copied.set(true);
                TimeoutFuture::new(1_200).await;
                copied.set(false);
            }
        });
    };

    view! {
        <div class="max-w-md p-4 flex flex-col gap-3">
            <h3 class="text-lg font-bold text-teal-300">"Pipeline"</h3>
            <div class="flex flex-col gap-3">
                <For
                    each=move || rows.get()
                    key=|r| r.id
                    children=move |r| {
                        let id = r.id;
                        view! {
                            <OpCard
                                id=id
                                tag=r.inst.tag.clone()
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
                    let tag = event_target_value(&ev);
                    if !tag.is_empty() { add_op(tag); }
                }
            >
                <option value="">"+ Add operation…"</option>
                {all_op_menu().into_iter().map(|(tag, label)| view! {
                    <option value=tag>{label}</option>
                }).collect_view()}
            </select>

            // ---- Run pipeline button ----
            <button
                class="bg-teal-600 hover:bg-teal-500 disabled:bg-slate-700 disabled:text-slate-500 text-white font-bold rounded-md px-4 py-2"
                prop:disabled=move || !can_run.get()
                on:click=move |_| on_run.run(())
            >
                "Run pipeline"
            </button>

            // ---- YAML preview ----
            <div class="mt-2 flex flex-col gap-2">
                <label class="flex items-center gap-3 cursor-pointer select-none">
                    <span class="text-sm font-bold text-teal-300">"Pipeline YAML"</span>
                    <span class="relative inline-block">
                        <input
                            type="checkbox"
                            class="peer sr-only"
                            prop:checked=move || show_debug_read.get()
                            on:change=move |ev| show_debug_write.set(event_target_checked(&ev))
                        />
                        <span
                            class="block w-9 h-5 rounded-full bg-slate-700 \
                                   peer-checked:bg-teal-600 transition-colors"
                        ></span>
                        <span
                            class="absolute left-0.5 top-0.5 w-4 h-4 rounded-full bg-slate-200 \
                                   transition-transform peer-checked:translate-x-4"
                        ></span>
                    </span>
                </label>

                {move || show_debug_read.get().then(|| view! {
                    <div class="flex flex-col gap-1">
                        <div class="flex justify-end">
                            <button
                                class="text-xs text-slate-400 hover:text-teal-300 px-2 py-1 \
                                       border border-slate-700 rounded w-20"
                                on:click=copy_yaml
                            >
                                {move || if copied.get() { "Copied!" } else { "Copy" }}
                            </button>
                        </div>
                        <pre class="text-xs bg-slate-950 text-slate-300 p-3 rounded overflow-x-auto">
                            {pipeline_yaml}
                        </pre>
                    </div>
                })}
            </div>
        </div>
    }
}
