pub(super) mod sliders;
use crate::op_instance::ParamValue;
use crate::{EditPayload, OpRow};
use leptos::prelude::*;
use pixelizer_core::op_schema::{ParamDescriptor, ParamKind, op_variants};
use sliders::{FloatSlider, IntSlider, decimals_for_step};

/// Read one param's current value from the live instance's bag, as f64.
/// Returns None if the row or key is absent.
fn read_num(rows: ReadSignal<Vec<OpRow>>, id: usize, key: &str) -> Option<f64> {
    rows.with(|rs| {
        rs.iter()
            .find(|r| r.id == id)
            .and_then(|r| r.inst.values.get(key))
            .and_then(ParamValue::as_num)
    })
}

fn param_widget(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    p: &'static ParamDescriptor,
) -> AnyView {
    let key = p.key; // &'static str: Copy
    let kind = p.kind; // ParamKind: Copy

    match kind {
        // -------- Float --------
        ParamKind::Float {
            default,
            min,
            max,
            step,
        } => {
            let value = Signal::derive(move || read_num(rows, id, key).unwrap_or(default as f64));
            let on_commit = Callback::new(move |raw: f64| {
                on_edit.run((id, key.to_string(), ParamValue::Num(raw)));
            });

            view! {
                <FloatSlider
                    label=p.label
                    value=value
                    min=min as f64 max=max as f64 step=step as f64
                    decimals=decimals_for_step(step as f64)
                    on_commit=on_commit
                />
            }
            .into_any()
        }

        // -------- Int --------
        ParamKind::Int { default, min, max } => {
            let value = Signal::derive(move || {
                read_num(rows, id, key)
                    .map(|n| n.round() as i64)
                    .unwrap_or(default)
            });
            let on_commit = Callback::new(move |raw: i64| {
                let clamped = raw.clamp(min, max);
                on_edit.run((id, key.to_string(), ParamValue::Num(clamped as f64)));
            });

            view! {
                <IntSlider
                    label=p.label
                    value=value
                    min=min max=max
                    on_commit=on_commit
                />
            }
            .into_any()
        }

        // -------- Bool --------
        // ParamKind::Bool { default } => bool_widget(id, rows, on_edit, default, key, p.label),
        ParamKind::Bool { default } => view! {
            <BoolWidget
                id=id
                rows=rows
                on_edit=on_edit
                default=default
                key=key
                label=p.label
            />
        }
        .into_any(),

        // Palette / Dither are not generic scalar params; palette_map has its
        // own config. If one appears here it's a routing bug, so surface it.
        ParamKind::Palette { .. } | ParamKind::Dither { .. } => view! {
            <p class="text-xs text-red-400 p-3">
                "param '"{p.key}"' is not a scalar; wrong config path"
            </p>
        }
        .into_any(),
    }
}

// pub fn bool_widget(
//     id: usize,
//     rows: ReadSignal<Vec<OpRow>>,
//     on_edit: Callback<EditPayload>,
//     default: bool,
//     key: &'static str,
//     label: &'static str,
// ) -> AnyView {
#[component]
pub fn BoolWidget(
    id: usize,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
    default: bool,
    key: &'static str,
    label: &'static str,
) -> impl IntoView {
    let value = Signal::derive(move || {
        rows.with(|rs| {
            rs.iter()
                .find(|r| r.id == id)
                .and_then(|r| r.inst.values.get(key))
                .and_then(ParamValue::as_bool)
        })
        .unwrap_or(default)
    });
    let on_commit = Callback::new(move |checked: bool| {
        on_edit.run((id, key.to_string(), ParamValue::Bool(checked)));
    });

    view! {
        <label class="flex items-center gap-2 text-xs text-slate-400 p-3">
            <input
                type="checkbox"
                prop:checked=move || value.get()
                on:change=move |ev| on_commit.run(event_target_checked(&ev))
            />
            {label}
        </label>
    }
    .into_any()
}

/// Render every descriptor param for this op's tag, reading/writing the bag.
pub fn generic_op_config(
    id: usize,
    tag: &str,
    rows: ReadSignal<Vec<OpRow>>,
    on_edit: Callback<EditPayload>,
) -> AnyView {
    let Some(variant) = op_variants().iter().find(|v| v.tag == tag) else {
        let tag = tag.to_string();
        return view! { <p class="text-xs text-red-400 p-3">"No descriptor for "{tag}</p> }
            .into_any();
    };

    variant
        .params
        .iter()
        .map(|p| param_widget(id, rows, on_edit, p))
        .collect_view()
        .into_any()
}
