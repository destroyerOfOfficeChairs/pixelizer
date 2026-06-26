use crate::Image;
use image::imageops::FilterType::Nearest;
use image::imageops::crop_imm;
use image::imageops::resize;
pub fn downsample(pixel_size: u32, mut image: Image) -> Image {
    let (x, y, width, height) = dimensions(pixel_size, image.width(), image.height());
    image = crop_imm(&mut image, x, y, width, height).to_image();
    resize(&mut image, width / pixel_size, height / pixel_size, Nearest)
}

fn dimensions(pixel_size: u32, width: u32, height: u32) -> (u32, u32, u32, u32) {
    let v_trim = height % pixel_size;
    let h_trim = width % pixel_size;
    let new_height = height - v_trim; // Always loses bottom-most row of original pixels for odd numbered height.
    let new_width = width - h_trim; // Always loses right-most column of original pixels for odd numbered width.
    (v_trim / 2, h_trim / 2, new_width, new_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_all_01() {
        let (x, y, width, height) = dimensions(10, 104, 104);
        assert_eq!((2, 2, 100, 100), (x, y, width, height));
    }

    #[test]
    fn trim_all_02() {
        let (x, y, width, height) = dimensions(10, 105, 105);
        assert_eq!((2, 2, 100, 100), (x, y, width, height));
    }
}
