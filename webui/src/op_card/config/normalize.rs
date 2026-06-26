use super::number_slider::NumberSlider;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

pub fn normalize_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let low = Signal::derive(move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Normalize { low, high: _ } => Some(low as f64),
                _ => None,
            })
            .unwrap_or(0.0)
    });

    let high = Signal::derive(move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Normalize { low: _, high } => Some(high as f64),
                _ => None,
            })
            .unwrap_or(0.0)
    });

    let on_low_commit = Callback::new(move |raw: f64| {
        let v = round2(raw as f32);
        on_edit.run((
            id,
            Box::new(move |op| {
                if let Operation::Normalize { low, high: _ } = op {
                    *low = v;
                }
            }),
        ));
    });

    let on_high_commit = Callback::new(move |raw: f64| {
        let v = round2(raw as f32);
        on_edit.run((
            id,
            Box::new(move |op| {
                if let Operation::Normalize { low: _, high } = op {
                    *high = v;
                }
            }),
        ));
    });

    view! {
        <NumberSlider
            label="low"
            value=low
            display=move |v: f64| format!("{:.2}", v)
            min=0.0 max=1.0 step=0.01
            on_commit=on_low_commit
        />
        <NumberSlider
            label="high"
            value=high
            display=move |v: f64| format!("{:.2}", v)
            min=0.0 max=1.0 step=0.01
            on_commit=on_high_commit
        />
    }
    .into_any()
}
