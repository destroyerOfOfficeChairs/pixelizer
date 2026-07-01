mod op_card;
mod pipeline_list;
mod viewport;

use leptos::prelude::*;
use pipeline_list::PipelineList;
use pixelizer_core::Operation;
use viewport::Viewport;

use pixelizer_core::Pipeline;

use crate::viewport::encode_to_data_url;

pub type EditPayload = (usize, Box<dyn Fn(&mut Operation)>);

#[derive(Clone)]
pub struct OpRow {
    id: usize,
    op: Operation,
}

pub struct Palettes {
    palettes: Vec<(String, Vec<String>)>,
}

impl Palettes {
    fn load() -> Self {
        let raw = include_str!("../palettes.yaml");
        let map: std::collections::HashMap<String, Vec<String>> =
            yaml_serde::from_str(raw).expect("palettes.yaml failed to parse");
        let mut palettes: Vec<(String, Vec<String>)> = map.into_iter().collect();
        palettes.sort_by(|a, b| a.0.cmp(&b.0));
        Palettes { palettes }
    }
}

#[component]
fn App() -> impl IntoView {
    let (rows, set_rows) = signal(vec![OpRow {
        id: 0,
        op: Operation::Downsample { pixel_size: 8 },
    }]);
    let source = RwSignal::new(None::<pixelizer_core::Image>);
    let output_url = RwSignal::new(None::<String>);

    // Run logic stays here — it owns source/output_url. Exposed as a callback.
    let on_run = Callback::new(move |_: ()| {
        let Some(img) = source.get() else { return };
        let ops: Vec<Operation> = rows.get().into_iter().map(|r| r.op).collect();
        let pipeline = Pipeline { operations: ops };
        // synchronous — UI freezes here for a few seconds. Known.
        match pixelizer_core::apply(&pipeline, img) {
            Ok(result) => output_url.set(Some(encode_to_data_url(&result))),
            Err(e) => leptos::logging::error!("pipeline failed: {e:?}"),
        }
    });

    // Whether a run is currently possible (no image = can't run).
    let can_run = Signal::derive(move || source.get().is_some());

    view! {
        <div class="flex gap-6 p-6 items-start">
            <div class="flex flex-col gap-3">
                <PipelineList
                    rows=rows
                    set_rows=set_rows
                    on_run=on_run
                    can_run=can_run
                />
            </div>
            <div class="flex-1">
                <Viewport source=source output_url=output_url/>
            </div>
        </div>
    }
}
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        provide_context(StoredValue::new(Palettes::load()));
        view! { <App/> }
    });
}
