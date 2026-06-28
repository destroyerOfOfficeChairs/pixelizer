mod op_card;
mod pipeline_list;

use leptos::prelude::*;
use pipeline_list::PipelineList;
use pixelizer_core::Operation;

use base64::{Engine, engine::general_purpose::STANDARD};
use pixelizer_core::image::{self, ImageFormat};
use pixelizer_core::{Image, Pipeline};
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

    view! {
        <div class="p-4 flex flex-col gap-3">
            <input type="file" accept="image/*" on:change=on_file
                class="text-sm text-slate-300"/>
            {move || output_url.get().map(|url| view! {
                <img src=url class="max-w-full border border-slate-700 rounded"/>
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
    let source = RwSignal::new(None::<pixelizer_core::Image>); // decoded RgbaImage
    let output_url = RwSignal::new(None::<String>); // PNG data URL for display
    let run = move |_| {
        let Some(img) = source.get() else { return };
        let ops: Vec<Operation> = rows.get().into_iter().map(|r| r.op).collect();
        let pipeline = Pipeline { operations: ops };
        // PHASE 1: synchronous — UI freezes here for a few seconds. Known.
        match pixelizer_core::apply(&pipeline, img) {
            Ok(result) => output_url.set(Some(encode_to_data_url(&result))),
            Err(e) => leptos::logging::error!("pipeline failed: {e:?}"),
        }
    };
    view! {
        <div class="flex gap-6 p-6 items-start">
            <div class="flex flex-col gap-3">
                <PipelineList rows=rows set_rows=set_rows/>
                <button
                    class="bg-teal-600 hover:bg-teal-500 disabled:bg-slate-700 disabled:text-slate-500 text-white font-bold rounded-md px-4 py-2"
                    prop:disabled=move || source.get().is_none()
                    on:click=run
                >
                    "Run pipeline"
                </button>
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
