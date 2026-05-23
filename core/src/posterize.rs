use crate::Image;
use crate::PixelizerError;

pub fn posterize(image: Image, levels: u32) -> Result<Image, PixelizerError> {
    if levels < 2 {
        return Err(PixelizerError::PosterizeError(
            "Posterize needs at least 2 levels per channel.".to_owned(),
        ));
    }

    let (w, h) = image.dimensions();
    let mut out = Image::new(w, h);
    let step = 255.0 / (levels - 1) as f32;

    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        out.put_pixel(
            x,
            y,
            image::Rgba([
                quantize_channel(r, step),
                quantize_channel(g, step),
                quantize_channel(b, step),
                a,
            ]),
        );
    }
    Ok(out)
}

fn quantize_channel(value: u8, step: f32) -> u8 {
    let v = value as f32;
    ((v / step).round() * step) as u8
}
