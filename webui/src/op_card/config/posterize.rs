use super::number_slider::NumberSlider;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::Operation;

pub fn posterize_config(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let value = Signal::derive(move || {
        rows.get()
            .iter()
            .find(|r| r.id == id)
            .and_then(|r| match r.op {
                Operation::Posterize { levels } => Some(levels as f64),
                _ => None,
            })
            .unwrap_or(0.0)
    });

    let on_commit = Callback::new(move |raw: f64| {
        let v = raw.round() as u32; // cast back to the field's real type
        on_edit.run((
            id,
            Box::new(move |op| {
                if let Operation::Posterize { levels } = op {
                    *levels = v;
                }
            }),
        ));
    });

    view! {
        <NumberSlider
            label="levels"
            value=value
            display=move |v: f64| format!("{}", v as u32)
            min=2.0 max=16.0 step=1.0
            on_commit=on_commit
        />
    }
    .into_any()
}
