use crate::{Image, PixelizerError, TrimMode};
use image::imageops::crop_imm;

pub fn trim_height(
    mode: TrimMode,
    mut image: Image,
    pixel_size: u32,
) -> Result<Image, PixelizerError> {
    let new_image_height: u32 = image.height() - (image.height() % pixel_size);
    let trim_amount: u32 = image.height() - new_image_height;

    let y_start: u32 = match mode {
        TrimMode::Top => trim_amount,
        TrimMode::Bottom => 0,
        TrimMode::Both => trim_amount / 2 + ((image.height() - trim_amount) % 2),
        _ => {
            return Err(PixelizerError::TrimError(
                "Left/Right does not make sense for trimming height.".to_owned(),
            ));
        }
    };

    let width = image.width();
    Ok(crop_imm(&mut image, 0, y_start, width, new_image_height).to_image())
}
