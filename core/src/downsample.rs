use crate::Image;
use image::imageops::FilterType::Nearest;
use image::imageops::resize;
pub fn downsample(mut image: Image, pixel_size: u32) -> Image {
    let new_image_width: u32 = image.width() / pixel_size;
    let new_image_height: u32 = image.height() / pixel_size;
    resize(&mut image, new_image_width, new_image_height, Nearest)
}
