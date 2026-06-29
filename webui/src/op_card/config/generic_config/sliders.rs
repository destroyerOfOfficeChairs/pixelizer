use leptos::prelude::*;

// ----------------------------------------------------------------------------
// Precision helper: decimals implied by a step value.
//   0.1  -> 1
//   0.01 -> 2
//   1.0  -> 0   (shouldn't be used for floats, but safe)
// Kept here (webui) rather than core: it's a display concern. Core owns `step`;
// the UI decides how to render it.
// ----------------------------------------------------------------------------
pub fn decimals_for_step(step: f64) -> usize {
    if step <= 0.0 || step >= 1.0 {
        return 0;
    }
    // Count fractional digits by scaling until the step is (near) integral.
    let mut d = 0;
    let mut s = step;
    while s < 1.0 && d < 10 {
        s *= 10.0;
        d += 1;
        if (s - s.round()).abs() < 1e-9 {
            break;
        }
    }
    d
}

// ----------------------------------------------------------------------------
// IntSlider — integer-valued. Number box + range, both in i64.
// ----------------------------------------------------------------------------
#[component]
pub fn IntSlider(
    label: &'static str,
    value: Signal<i64>,
    min: i64,
    max: i64,
    /// Called with the parsed i64. Caller stores it.
    on_commit: Callback<i64>,
) -> impl IntoView {
    let shown = move || value.get().to_string();

    view! {
        <label class="text-xs text-slate-400 block p-3">
            {label}": "
            <input
                type="number"
                min=min as f64 max=max as f64 step=1.0
                prop:value=shown
                on:change=move |ev| {
                    let raw: i64 = event_target_value(&ev).parse().unwrap_or(min);
                    on_commit.run(raw.clamp(min, max));
                }
            />
            <input
                type="range"
                min=min as f64 max=max as f64 step=1.0
                prop:value=shown
                class="w-full accent-teal-500"
                on:input=move |ev| {
                    let raw: i64 = event_target_value(&ev)
                        .parse::<f64>()
                        .map(|f| f.round() as i64)
                        .unwrap_or(min);
                    on_commit.run(raw.clamp(min, max));
                }
            />
        </label>
    }
}

fn snap(raw: f64, min: f64, max: f64, step: f64, decimals: usize) -> f64 {
    let clamped = raw.clamp(min, max);
    if step <= 0.0 {
        return clamped;
    }
    let snapped = (clamped / step).round() * step;
    let factor = 10f64.powi(decimals as i32);
    (snapped * factor).round() / factor
}

// ----------------------------------------------------------------------------
// FloatSlider — float-valued. Number box + range, in f64, with explicit
// display precision (from decimals_for_step at the call site).
// ----------------------------------------------------------------------------
#[component]
pub fn FloatSlider(
    label: &'static str,
    value: Signal<f64>,
    min: f64,
    max: f64,
    step: f64,
    /// How many decimal places to display.
    decimals: usize,
    /// Called with the parsed f64. Caller stores it.
    on_commit: Callback<f64>,
) -> impl IntoView {
    let shown = move || format!("{:.*}", decimals, value.get());

    view! {
        <label class="text-xs text-slate-400 block p-3">
            {label}": "
            <input
                type="number"
                min=min max=max step=step
                prop:value=shown
                on:change=move |ev| {
                    let raw: f64 = event_target_value(&ev).parse().unwrap_or(min);
                    on_commit.run(snap(raw, min, max, step, decimals));
                }
            />
            <input
                type="range"
                min=min max=max step=step
                prop:value=shown
                class="w-full accent-teal-500"
                on:input=move |ev| {
                    let raw: f64 = event_target_value(&ev).parse().unwrap_or(min);
                    on_commit.run(snap(raw, min, max, step, decimals));
                }
            />
        </label>
    }
}
