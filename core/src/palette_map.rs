use crate::DitherConfig;
use crate::DitherKind;
use crate::Image;
use std::collections::HashMap;

use crate::color_utils::PaletteData;
use crate::color_utils::nearest_oklab;
use crate::color_utils::prepare_palette;
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

const JJN: &[(i32, i32, f32)] = &[
    (1, 0, 7.0 / 48.0),
    (2, 0, 5.0 / 48.0),
    (-2, 1, 3.0 / 48.0),
    (-1, 1, 5.0 / 48.0),
    (0, 1, 7.0 / 48.0),
    (1, 1, 5.0 / 48.0),
    (2, 1, 3.0 / 48.0),
    (-2, 2, 1.0 / 48.0),
    (-1, 2, 3.0 / 48.0),
    (0, 2, 5.0 / 48.0),
    (1, 2, 3.0 / 48.0),
    (2, 2, 1.0 / 48.0),
];

pub fn palette_map(
    image: Image,
    colors: &[String],
    dither: Option<DitherConfig>,
) -> Result<Image, crate::PixelizerError> {
    if let Some(dither_config) = dither {
        return palette_map_dithered(&image, colors, dither_config);
    };

    let PaletteData {
        rgb,
        lab,
        linear: _,
        max_per_channel: _,
    } = prepare_palette(colors)?;

    let (w, h) = image.dimensions();
    let mut out = Image::new(w, h);

    let mut cache: HashMap<[u8; 3], usize> = HashMap::new();

    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let idx = *cache
            .entry([r, g, b])
            .or_insert_with(|| nearest_oklab(&lab, rgb_to_oklab(r, g, b)));
        let [pr, pg, pb] = rgb[idx];
        out.put_pixel(x, y, image::Rgba([pr, pg, pb, a]));
    }
    Ok(out)
}

pub fn palette_map_dithered(
    image: &Image,
    colors: &[String],
    dither_config: DitherConfig,
) -> Result<Image, crate::PixelizerError> {
    let PaletteData {
        rgb,
        lab,
        linear,
        max_per_channel,
    } = prepare_palette(colors)?;

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

    let alg = match dither_config.kind {
        DitherKind::Atkinson => ATKINSON,
        DitherKind::FloydSteinberg => FLOYD_STEINBERG,
        DitherKind::JJN => JJN,
    };

    let clamp = dither_config.clamp;
    let error_damping = dither_config.bleed;

    for y in 0..h {
        let ltr = y % 2 == 0;
        let sign = if ltr { 1 } else { -1 };
        let xs: Box<dyn Iterator<Item = u32>> = if ltr {
            Box::new(0..w)
        } else {
            Box::new((0..w).rev())
        };

        for x in xs {
            let pixel = if clamp {
                let p = buf[idx(x, y)];
                [
                    p[0].clamp(0.0, max_per_channel[0]),
                    p[1].clamp(0.0, max_per_channel[1]),
                    p[2].clamp(0.0, max_per_channel[2]),
                ]
            } else {
                buf[idx(x, y)]
            };
            let (pal_idx, error) = quantize(&lab, &linear, pixel, error_damping);
            let [pr, pg, pb] = rgb[pal_idx];
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
