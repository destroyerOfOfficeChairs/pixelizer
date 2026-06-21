use pixelizer_core::{Pipeline, apply, image};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Result handed back to JS: raw RGBA8 plus dimensions so the
/// canvas can build an ImageData directly.
#[wasm_bindgen]
pub struct PixelizeResult {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl PixelizeResult {
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
    // Returns the RGBA bytes. `getter_with_clone` because Vec<u8> isn't Copy.
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

/// `image_bytes`: the raw bytes of an uploaded PNG/JPEG file.
/// `pipeline_json`: a JSON string matching pixelizer_core::Pipeline.
#[wasm_bindgen]
pub fn pixelize(image_bytes: &[u8], pipeline_json: &str) -> Result<PixelizeResult, JsError> {
    let pipeline: Pipeline = serde_json::from_str(pipeline_json)
        .map_err(|e| JsError::new(&format!("bad pipeline JSON: {e}")))?;

    let img = image::load_from_memory(image_bytes)
        .map_err(|e| JsError::new(&format!("could not decode image: {e}")))?
        .to_rgba8();

    let out =
        apply(&pipeline, img).map_err(|e| JsError::new(&format!("pipeline failed: {e:?}")))?;

    let (width, height) = out.dimensions();
    Ok(PixelizeResult {
        width,
        height,
        data: out.into_raw(),
    })
}
