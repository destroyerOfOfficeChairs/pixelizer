use crate::Image;
use image::imageops::FilterType::Nearest;
use image::imageops::resize;
pub fn upscale(mut image: Image, factor: u32) -> Image {
    let new_image_width: u32 = image.width() * factor;
    let new_image_height: u32 = image.height() * factor;
    resize(&mut image, new_image_width, new_image_height, Nearest)
}
