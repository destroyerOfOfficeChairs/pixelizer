use super::number_slider::NumberSlider;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

pub fn blur_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let value = Signal::derive(move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Blur { sigma } => Some(sigma as f64),
                _ => None,
            })
            .unwrap_or(0.0)
    });

    let on_commit = Callback::new(move |raw: f64| {
        let v = round1(raw as f32);
        on_edit.run((
            id,
            Box::new(move |op| {
                if let Operation::Blur { sigma } = op {
                    *sigma = v;
                }
            }),
        ));
    });

    view! {
        <NumberSlider
            label="sigma"
            value=value
            display=move |v: f64| format!("{:.1}", v)
            min=0.0 max=10.0 step=0.1
            on_commit=on_commit
        />
    }
    .into_any()
}
