pub use image;
mod color_utils;
mod downsample;
mod palette_map;
mod trim_height;
mod trim_width;
mod upscale;
use downsample::downsample;
use palette_map::palette_map;
use trim_height::trim_height;
use trim_width::trim_width;
use upscale::upscale;

pub type Image = image::RgbaImage;

#[derive(Debug)]
pub enum PixelizerError {
    TrimError(String),
    OrderError(String),
    HexParseError(String),
    NoColorsError(String),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pipeline {
    pub operations: Vec<Operation>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
    PixelSize {
        size: u32,
    },
    TrimHeight {
        mode: TrimMode,
    },
    TrimWidth {
        mode: TrimMode,
    },
    Downsample,
    PaletteMap {
        colors: Vec<String>,
        #[serde(default)]
        dither: Option<DitherConfig>,
    },
    Upscale {
        factor: u32,
    },
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub struct DitherConfig {
    pub kind: DitherKind,
    #[serde(default)]
    pub clamp: Option<bool>,
    pub bleed: Option<f32>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DitherKind {
    FloydSteinberg,
    Atkinson,
    JJN,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    Top,
    Bottom,
    Left,
    Right,
    Both,
}

pub fn apply(pipeline: &Pipeline, mut image: Image) -> Result<Image, PixelizerError> {
    let mut pixel_size: u32 = 1;

    for (i, op) in pipeline.operations.iter().enumerate() {
        match op {
            Operation::PixelSize { size } => {
                if i != 0 {
                    return Err(PixelizerError::OrderError(
                        "Setting pixel size must be the first operation.".to_owned(),
                    ));
                }
                pixel_size = *size;
            }
            Operation::TrimHeight { mode } => {
                image = trim_height(*mode, image, pixel_size)?;
            }
            Operation::TrimWidth { mode } => {
                image = trim_width(*mode, image, pixel_size)?;
            }
            Operation::Downsample => image = downsample(image, pixel_size),
            Operation::PaletteMap { colors, dither } => {
                image = palette_map(image, colors, *dither)?
            }
            Operation::Upscale { factor } => image = upscale(image, *factor),
        }
    }
    Ok(image)
}
