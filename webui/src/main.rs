mod op_card;
mod pipeline_list;

use leptos::prelude::*;
use pipeline_list::PipelineList;
use pixelizer_core::Operation;

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
fn app() -> impl IntoView {
    view! {
        <div>
            <div> // This will be on the left side of the screen
                <PipelineList/>
            </div>
            <div> // And this will be on the right
                <Viewport/> // Gotta implement this
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        provide_context(StoredValue::new(Palettes::load()));
        view! { <PipelineList/> }
    });
}
