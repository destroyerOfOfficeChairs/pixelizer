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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(pixels: Vec<(u8, u8, u8)>) -> Image {
        let mut img = Image::new(pixels.len() as u32, 1);
        for (i, (r, g, b)) in pixels.into_iter().enumerate() {
            img.put_pixel(i as u32, 0, image::Rgba([r, g, b, 255]));
        }
        img
    }

    #[test]
    fn posterize_rejects_levels_below_2() {
        let img = make_test_image(vec![(128, 128, 128)]);
        assert!(posterize(img, 1).is_err());
        // levels=0 should also be rejected.
        let img = make_test_image(vec![(128, 128, 128)]);
        assert!(posterize(img, 0).is_err());
    }

    #[test]
    fn posterize_levels_2_produces_binary_channels() {
        // levels=2 means each channel snaps to 0 or 255.
        let img = make_test_image(vec![
            (10, 10, 10),    // dark → 0
            (100, 100, 100), // mid-low → 0 or 255 (boundary)
            (200, 200, 200), // mid-high → 255
            (250, 250, 250), // bright → 255
        ]);
        let out = posterize(img, 2).unwrap();
        for x in 0..4 {
            let p = out.get_pixel(x, 0).0;
            for c in 0..3 {
                assert!(
                    p[c] == 0 || p[c] == 255,
                    "pixel {} channel {} = {}",
                    x,
                    c,
                    p[c]
                );
            }
        }
    }

    #[test]
    fn posterize_preserves_alpha() {
        let mut img = Image::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([128, 128, 128, 99]));
        let out = posterize(img, 4).unwrap();
        assert_eq!(out.get_pixel(0, 0).0[3], 99);
    }

    #[test]
    fn posterize_preserves_dimensions() {
        let img = Image::new(50, 30);
        let out = posterize(img, 4).unwrap();
        assert_eq!(out.dimensions(), (50, 30));
    }
}
