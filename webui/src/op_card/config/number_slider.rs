use leptos::prelude::*;

#[component]
pub fn NumberSlider(
    label: &'static str,
    // Reactive current value, already in f64. Caller derives this from rows.
    value: Signal<f64>,
    // How to render the value as text (e.g. one-decimal vs integer).
    #[prop(into)] display: Callback<f64, String>,
    min: f64,
    max: f64,
    step: f64,
    // Called with the raw f64 from the input; caller quantizes/casts/stores.
    on_commit: Callback<f64>,
) -> impl IntoView {
    // Shared display string for both inputs.
    let shown = move || display.run(value.get());

    view! {
        <label class="text-xs text-slate-400 block p-3">
            {label}": "
            <input
                type="number"
                min=min max=max step=step
                prop:value=shown
                on:change=move |ev| {
                    let raw: f64 = event_target_value(&ev).parse().unwrap_or(min);
                    on_commit.run(raw);
                }
            />
            <input
                type="range"
                min=min max=max step=step
                prop:value=shown
                class="w-full accent-teal-500"
                on:input=move |ev| {
                    let raw: f64 = event_target_value(&ev).parse().unwrap_or(min);
                    on_commit.run(raw);
                }
            />
        </label>
    }
}
