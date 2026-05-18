pub use image;
pub type Image = image::RgbaImage;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pipeline {
    pub operations: Vec<Operation>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
    PixelSize { size: u32 },
    TrimHeight { mode: TrimMode },
    TrimWidth { mode: TrimMode },
    Downsample,
    PaletteMap { colors: Vec<String> }, // hex strings for now
    Upscale { factor: u32 },
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    Top,
    Bottom,
    Both,
}

pub fn apply(_pipeline: &Pipeline, image: &Image) -> Image {
    image.clone() // stub: just returns the input
}
