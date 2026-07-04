pub use image;
mod blur;
mod color_utils;
mod downsample;
mod normalize;
pub mod op_schema;
mod palette_map;
mod posterize;
mod upscale;
use blur::blur;
use downsample::downsample;
use normalize::normalize;
use palette_map::palette_map;
use posterize::posterize;
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
    HexParseError(String),
    NoColorsError(String),
    PosterizeError(String),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pipeline {
    pub operations: Vec<Operation>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
    Downsample {
        pixel_size: u32,
    },
    PaletteMap {
        colors: Vec<String>,
        #[serde(default)]
        dither: Option<DitherConfig>,
        #[serde(default = "preserve_alpha_default")]
        preserve_alpha: bool,
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

fn preserve_alpha_default() -> bool {
    true
}

fn default_low_percentile() -> f32 {
    0.01
}
fn default_high_percentile() -> f32 {
    0.99
} // clip brightest 1%

pub fn apply(pipeline: &Pipeline, mut image: Image) -> Result<Image, PixelizerError> {
    for op in &pipeline.operations {
        match op {
            Operation::Downsample { pixel_size } => image = downsample(*pixel_size, image),
            Operation::PaletteMap {
                colors,
                dither,
                preserve_alpha,
            } => image = palette_map(image, colors, *dither, *preserve_alpha)?,
            Operation::Upscale { factor } => image = upscale(image, *factor),
            Operation::Posterize { levels } => image = posterize(image, *levels)?,
            Operation::Blur { sigma } => image = blur(image, *sigma),
            Operation::Normalize { low, high } => image = normalize(image, *low, *high),
        }
    }
    Ok(image)
}
