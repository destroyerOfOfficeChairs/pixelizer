use crate::{Image, TrimMode};
use image::imageops::FilterType::Nearest;
use image::imageops::crop_imm;
use image::imageops::resize;
pub fn downsample(pixel_size: u32, trim: TrimMode, mut image: Image) -> Image {
    let (x, y, width, height) = dimensions(pixel_size, trim, image.width(), image.height());
    if trim != TrimMode::TrimNone {
        image = crop_imm(&mut image, x, y, width, height).to_image();
    }
    resize(&mut image, width / pixel_size, height / pixel_size, Nearest)
}

fn dimensions(pixel_size: u32, trim: TrimMode, width: u32, height: u32) -> (u32, u32, u32, u32) {
    let v_trim = height % pixel_size;
    let h_trim = width % pixel_size;
    let new_height = height - v_trim; // Always loses bottom-most row of original pixels for odd numbered height.
    let new_width = width - h_trim; // Always loses right-most column of original pixels for odd numbered width.
    match trim {
        TrimMode::TrimTop => (0, v_trim, width, new_height),
        TrimMode::TrimBottom => (0, 0, width, new_height),
        TrimMode::TrimLeft => (h_trim, 0, new_width, height),
        TrimMode::TrimRight => (0, 0, new_width, height),
        TrimMode::TrimTopAndLeft => (h_trim, v_trim, new_width, new_height),
        TrimMode::TrimTopAndRight => (0, v_trim, new_width, new_height),
        TrimMode::TrimBottomAndLeft => (h_trim, 0, new_width, new_height),
        TrimMode::TrimBottomAndRight => (0, 0, new_width, new_height),
        TrimMode::TrimVertical => (0, v_trim / 2, width, new_height),
        TrimMode::TrimHorizontal => (h_trim / 2, 0, new_width, height),
        TrimMode::TrimAll => (v_trim / 2, h_trim / 2, new_width, new_height),
        TrimMode::TrimNone => (0, 0, width, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_top_01() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimTop, 100, 105);
        assert_eq!((0, 5, 100, 100), (x, y, width, height));
    }

    #[test]
    fn trim_top_02() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimTop, 101, 106);
        assert_eq!((0, 6, 101, 100), (x, y, width, height));
    }

    #[test]
    fn trim_bottom_01() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimBottom, 100, 105);
        assert_eq!((0, 0, 100, 100), (x, y, width, height));
    }

    #[test]
    fn trim_bottom_02() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimBottom, 101, 106);
        assert_eq!((0, 0, 101, 100), (x, y, width, height));
    }

    #[test]
    fn trim_all_01() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimAll, 104, 104);
        assert_eq!((2, 2, 100, 100), (x, y, width, height));
    }

    #[test]
    fn trim_all_02() {
        let (x, y, width, height) = dimensions(10, TrimMode::TrimAll, 105, 105);
        assert_eq!((2, 2, 100, 100), (x, y, width, height));
    }
}
