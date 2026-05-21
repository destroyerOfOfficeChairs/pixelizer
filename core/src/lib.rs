pub use image;
use image::imageops;

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
    PaletteMap { colors: Vec<String> },
    Upscale { factor: u32 },
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

pub fn apply(pipeline: &Pipeline, image: &Image) -> Image {
    let mut image: Image = image.clone();
    let mut pixel_size: u32 = 1;

    for (i, op) in pipeline.operations.iter().enumerate() {
        match op {
            Operation::PixelSize { size } => {
                if i != 0 {
                    eprintln!("Setting pixel size must be the first operation.");
                    std::process::exit(1);
                }
                pixel_size = *size;
            }
            Operation::TrimHeight { mode } => {
                image = trim_height(*mode, image, pixel_size);
            }
            Operation::TrimWidth { mode } => {
                image = trim_width(*mode, image, pixel_size);
            }
            Operation::Downsample => image = downsample(image, pixel_size),
            Operation::PaletteMap { colors: _ } => println!("palette map"),
            Operation::Upscale { factor } => image = upscale(image, *factor),
        }
    }
    image
}

fn trim_height(mode: TrimMode, mut image: Image, pixel_size: u32) -> Image {
    let new_image_height: u32 = image.height() - (image.height() % pixel_size);
    let trim_amount: u32 = image.height() - new_image_height;

    let mut y_start: u32 = match mode {
        TrimMode::Top => trim_amount,
        TrimMode::Bottom => 0,
        TrimMode::Both => trim_amount / 2, // rounds down; bottom keeps the extra row
        _ => 0,
    };

    if trim_amount % 2 != 0 {
        y_start += 1;
    };

    let width = image.width();
    imageops::crop_imm(&mut image, 0, y_start, width, new_image_height).to_image()
}

fn trim_width(mode: TrimMode, mut image: Image, pixel_size: u32) -> Image {
    let new_image_width: u32 = image.width() - (image.width() % pixel_size);
    let trim_amount: u32 = image.width() - new_image_width;

    let mut x_start: u32 = match mode {
        TrimMode::Left => trim_amount,
        TrimMode::Right => 0,
        TrimMode::Both => trim_amount / 2,
        _ => 0,
    };

    if trim_amount % 2 != 0 {
        x_start += 1;
    };

    let height = image.height();
    imageops::crop_imm(&mut image, x_start, 0, new_image_width, height).to_image()
}

fn downsample(mut image: Image, pixel_size: u32) -> Image {
    let new_image_width: u32 = image.width() / pixel_size;
    let new_image_height: u32 = image.height() / pixel_size;
    imageops::resize(
        &mut image,
        new_image_width,
        new_image_height,
        imageops::FilterType::Nearest,
    )
}

fn upscale(mut image: Image, factor: u32) -> Image {
    let new_image_width: u32 = image.width() * factor;
    let new_image_height: u32 = image.height() * factor;
    imageops::resize(
        &mut image,
        new_image_width,
        new_image_height,
        imageops::FilterType::Nearest,
    )
}
