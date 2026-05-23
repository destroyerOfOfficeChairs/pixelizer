use crate::Image;
use image::imageops;

pub fn blur(image: Image, sigma: f32) -> Image {
    imageops::blur(&image, sigma)
}
