mod op_card;
mod pipeline_list;

use leptos::prelude::*;
use pipeline_list::PipelineList;
use pixelizer_core::Operation;

use base64::{Engine, engine::general_purpose::STANDARD};
use pixelizer_core::Pipeline;
use pixelizer_core::image::{self, ImageFormat};
use std::io::Cursor;

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

fn decode(bytes: &[u8]) -> Option<pixelizer_core::Image> {
    image::load_from_memory(bytes)
        .ok()
        .map(|img| img.to_rgba8())
}

fn encode_to_data_url(img: &pixelizer_core::Image) -> String {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .expect("PNG encode");
    format!("data:image/png;base64,{}", STANDARD.encode(&buf))
}

#[component]
fn Viewport(
    source: RwSignal<Option<pixelizer_core::Image>>,
    output_url: RwSignal<Option<String>>,
) -> impl IntoView {
    let on_file = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        if let Some(file_list) = input.files() {
            if let Some(file) = file_list.get(0) {
                let gloo_file = gloo_file::File::from(file);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(bytes) = gloo_file::futures::read_as_bytes(&gloo_file).await {
                        if let Some(img) = decode(&bytes) {
                            source.set(Some(img));
                        }
                    }
                });
            }
        }
    };

    // What to display: prefer the processed output; fall back to the decoded
    // source so the image shows immediately on upload, before any run.
    // Returns an Option<String> data URL. Re-runs only when output_url or
    // source changes — so the source re-encode happens once per new image,
    // not on every render.
    let display_url = move || {
        output_url
            .get()
            .or_else(|| source.get().as_ref().map(encode_to_data_url))
    };

    view! {
        <div class="p-4 flex flex-col gap-3">
            <input type="file" accept="image/*" on:change=on_file
                class="text-sm text-slate-300"/>
            {move || display_url().map(|url| view! {
                <img
                    src=url
                    class="max-w-full max-h-[80vh] object-contain border border-slate-700 rounded [image-rendering:pixelated]"
                />
            })}
        </div>
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
