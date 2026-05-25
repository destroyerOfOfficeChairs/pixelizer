pub use image;
mod blur;
mod color_utils;
mod downsample;
mod normalize;
mod palette_map;
mod posterize;
mod trim_height;
mod trim_width;
mod upscale;
use blur::blur;
use downsample::downsample;
use normalize::normalize;
use palette_map::palette_map;
use posterize::posterize;
use trim_height::trim_height;
use trim_width::trim_width;
use upscale::upscale;

pub type Image = image::RgbaImage;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(tag = "algorithm", rename_all = "snake_case")]
pub enum DitherConfig {
    FloydSteinberg {
        #[serde(default = "default_bleed")]
        bleed: f32,
        #[serde(default)]
        clamp: bool,
    },
    Atkinson {
        #[serde(default = "default_bleed")]
        bleed: f32,
        #[serde(default)]
        clamp: bool,
    },
    #[serde(rename = "jjn")]
    Jjn {
        #[serde(default = "default_bleed")]
        bleed: f32,
        #[serde(default)]
        clamp: bool,
    },
    Bayer4 {
        #[serde(default = "default_strength")]
        strength: f32,
    },
    Bayer8 {
        #[serde(default = "default_strength")]
        strength: f32,
    },
}

fn default_bleed() -> f32 {
    1.0
}
fn default_strength() -> f32 {
    32.0
}

#[derive(Debug)]
pub enum PixelizerError {
    TrimError(String),
    OrderError(String),
    HexParseError(String),
    NoColorsError(String),
    PosterizeError(String),
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
    Posterize {
        levels: u32,
    },
    Blur {
        sigma: f32,
    },
    Normalize {
        #[serde(default = "default_low_percentile")]
        low: f32,
        #[serde(default = "default_high_percentile")]
        high: f32,
    },
}

fn default_low_percentile() -> f32 {
    0.01
}
fn default_high_percentile() -> f32 {
    0.99
} // clip brightest 1%

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
            Operation::Posterize { levels } => image = posterize(image, *levels)?,
            Operation::Blur { sigma } => image = blur(image, *sigma),
            Operation::Normalize { low, high } => image = normalize(image, *low, *high),
        }
    }
    Ok(image)
}
