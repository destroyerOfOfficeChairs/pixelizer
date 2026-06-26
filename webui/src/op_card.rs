mod op_config_view;
use crate::{OpRow, op_label};
use leptos::html;
use leptos::prelude::*;
use op_config_view::op_config_view;
use pixelizer_core::Operation;
type EditPayload = (usize, Box<dyn Fn(&mut Operation)>);
// ---- OpCard: one card. Top bar + collapsible animated settings area. ----
#[component]
pub fn OpCard(
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
