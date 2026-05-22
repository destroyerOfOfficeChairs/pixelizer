use crate::{Image, PixelizerError, TrimMode};
use image::imageops::crop_imm;

pub fn trim_width(
    mode: TrimMode,
    mut image: Image,
    pixel_size: u32,
) -> Result<Image, PixelizerError> {
    let new_image_width: u32 = image.width() - (image.width() % pixel_size);
    let trim_amount: u32 = image.width() - new_image_width;

    let x_start: u32 = match mode {
        TrimMode::Left => trim_amount,
        TrimMode::Right => 0,
        TrimMode::Both => trim_amount / 2 + ((image.width() - trim_amount) % 2),
        _ => {
            return Err(PixelizerError::TrimError(
                "Top/Bottom does not make sense for trimming width.".to_owned(),
            ));
        }
    };

    let height = image.height();
    Ok(crop_imm(&mut image, x_start, 0, new_image_width, height).to_image())
}
