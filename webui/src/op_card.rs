mod config;
use crate::{EditPayload, OpRow};
use config::op_config_view;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{UseElementSizeReturn, use_element_size};
use pixelizer_core::Operation;

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

// ---- OpCard: one card. Top bar + collapsible animated settings area. ----
#[component]
pub fn OpCard(
    id: usize,
    op: Operation,
    rows: ReadSignal<Vec<OpRow>>,
    on_move: Callback<i32>,
    on_remove: Callback<()>,
    on_edit: Callback<EditPayload>,
) -> impl IntoView {
    let (open, set_open) = signal(true);

    // A handle to the settings-content div, to measure its height.
    let content_ref: NodeRef<html::Div> = NodeRef::new();

    let label = op_label(&op);

    let UseElementSizeReturn { height, .. } = use_element_size(content_ref);

    let max_height = move || {
        if open.get() {
            format!("{}px", height.get())
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
                // Inner content is what is measured. Its natural height is the target.
                <div node_ref=content_ref>
                    <div class="p-3">
                        {op_config_view(id, &op, rows, on_edit)}
                    </div>
                </div>
            </div>
        </div>
    }
}
