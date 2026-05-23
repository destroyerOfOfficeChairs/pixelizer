use crate::DitherKind;
use crate::Image;
use crate::PixelizerError::NoColorsError;
use std::collections::HashMap;

use crate::color_utils::Oklab;
use crate::color_utils::nearest_oklab;
use crate::color_utils::parse_hex;
use crate::color_utils::quantize;
use crate::color_utils::rgb_to_oklab;
use crate::color_utils::srgb_to_linear;

const FLOYD_STEINBERG: &[(i32, i32, f32)] = &[
    (1, 0, 7.0 / 16.0),
    (-1, 1, 3.0 / 16.0),
    (0, 1, 5.0 / 16.0),
    (1, 1, 1.0 / 16.0),
];

const ATKINSON: &[(i32, i32, f32)] = &[
    (1, 0, 1.0 / 8.0),
    (2, 0, 1.0 / 8.0),
    (-1, 1, 1.0 / 8.0),
    (0, 1, 1.0 / 8.0),
    (1, 1, 1.0 / 8.0),
    (0, 2, 1.0 / 8.0),
];

pub fn palette_map(
    image: Image,
    colors: &[String],
    dither: Option<DitherKind>,
) -> Result<Image, crate::PixelizerError> {
    if let Some(dither_algorithm) = dither {
        return palette_map_dithered(&image, colors, dither_algorithm);
    };

    let palette_rgb: Vec<[u8; 3]> = colors
        .iter()
        .map(|s| parse_hex(s))
        .collect::<Result<_, _>>()?;

    if palette_rgb.is_empty() {
        return Err(NoColorsError(
            "There are no colors in the palette.".to_owned(),
        ));
    }

    // Precompute OkLab once per palette entry.
    let palette_lab: Vec<Oklab> = palette_rgb
        .iter()
        .map(|c| rgb_to_oklab(c[0], c[1], c[2]))
        .collect();

    let (w, h) = image.dimensions();
    let mut out = Image::new(w, h);

    let mut cache: HashMap<[u8; 3], usize> = HashMap::new();

    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let idx = *cache
            .entry([r, g, b])
            .or_insert_with(|| nearest_oklab(&palette_lab, rgb_to_oklab(r, g, b)));
        let [pr, pg, pb] = palette_rgb[idx];
        out.put_pixel(x, y, image::Rgba([pr, pg, pb, a]));
    }
    Ok(out)
}

pub fn palette_map_dithered(
    image: &Image,
    colors: &[String],
    dither_algorithm: DitherKind,
) -> Result<Image, crate::PixelizerError> {
    let palette_rgb: Vec<[u8; 3]> = colors
        .iter()
        .map(|s| parse_hex(s))
        .collect::<Result<_, _>>()?;

    if palette_rgb.is_empty() {
        return Err(NoColorsError(
            "There are no colors in the palette.".to_owned(),
        ));
    }

    let palette_lab: Vec<Oklab> = palette_rgb
        .iter()
        .map(|c| rgb_to_oklab(c[0], c[1], c[2]))
        .collect();

    // Precompute palette in linear RGB for error math.
    let palette_linear: Vec<[f32; 3]> = palette_rgb
        .iter()
        .map(|c| {
            [
                srgb_to_linear(c[0]),
                srgb_to_linear(c[1]),
                srgb_to_linear(c[2]),
            ]
        })
        .collect();

    let (w, h) = image.dimensions();

    // Working buffer in LINEAR light, not sRGB.
    let mut buf: Vec<[f32; 3]> = image
        .pixels()
        .map(|p| {
            [
                srgb_to_linear(p.0[0]),
                srgb_to_linear(p.0[1]),
                srgb_to_linear(p.0[2]),
            ]
        })
        .collect();

    let alpha: Vec<u8> = image.pixels().map(|p| p.0[3]).collect();
    let mut out = Image::new(w, h);

    let idx = |x: u32, y: u32| (y * w + x) as usize;

    // There's probably some more elegant way to do this.
    let alg = match dither_algorithm {
        DitherKind::Atkinson => ATKINSON,
        DitherKind::FloydSteinberg => FLOYD_STEINBERG,
    };

    for y in 0..h {
        let ltr = y % 2 == 0;
        let sign = if ltr { 1 } else { -1 };
        let xs: Box<dyn Iterator<Item = u32>> = if ltr {
            Box::new(0..w)
        } else {
            Box::new((0..w).rev())
        };

        for x in xs {
            let (pal_idx, error) = quantize(&palette_lab, &palette_linear, buf[idx(x, y)]);
            let [pr, pg, pb] = palette_rgb[pal_idx];
            out.put_pixel(x, y, image::Rgba([pr, pg, pb, alpha[idx(x, y)]]));

            for &(dx, dy, weight) in alg {
                let nx = x as i32 + dx * sign;
                let ny = y as i32 + dy;
                if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                    continue;
                }
                let p = &mut buf[idx(nx as u32, ny as u32)];
                for c in 0..3 {
                    p[c] += error[c] * weight;
                }
            }
        }
    }
    Ok(out)
}
