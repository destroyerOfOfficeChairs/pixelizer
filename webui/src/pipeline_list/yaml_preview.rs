use crate::OpRow;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use pixelizer_core::Pipeline;

#[component]
pub fn YamlPreview(rows: ReadSignal<Vec<OpRow>>) -> impl IntoView {
    let show_debug = signal(false);
    let (show_debug_read, show_debug_write) = show_debug;
    let copied = RwSignal::new(false);

    // Builds the "Pipeline YAML" preview.
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

    // Copy to clipboard, then flash "Copied!" for 1.2s.
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
    }
}
