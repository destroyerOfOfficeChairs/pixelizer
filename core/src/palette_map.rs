use crate::Image;
use crate::PixelizerError::NoColorsError;
use std::collections::HashMap;

use crate::color_utils::Oklab;
use crate::color_utils::linear_to_srgb;
use crate::color_utils::nearest_oklab;
use crate::color_utils::parse_hex;
use crate::color_utils::rgb_to_oklab;
use crate::color_utils::srgb_to_linear;
use crate::color_utils::srgb_to_linear_f32;

pub fn palette_map(
    image: Image,
    colors: &[String],
    dither: bool,
) -> Result<Image, crate::PixelizerError> {
    if dither {
        return palette_map_dithered(&image, colors);
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
                srgb_to_linear_f32(c[0]),
                srgb_to_linear_f32(c[1]),
                srgb_to_linear_f32(c[2]),
            ]
        })
        .collect();

    let (w, h) = image.dimensions();

    // Working buffer in LINEAR light, not sRGB.
    let mut buf: Vec<[f32; 3]> = image
        .pixels()
        .map(|p| {
            [
                srgb_to_linear_f32(p.0[0]),
                srgb_to_linear_f32(p.0[1]),
                srgb_to_linear_f32(p.0[2]),
            ]
        })
        .collect();

    let alpha: Vec<u8> = image.pixels().map(|p| p.0[3]).collect();
    let mut out = Image::new(w, h);

    let idx = |x: u32, y: u32| (y * w + x) as usize;

    for y in 0..h {
        let ltr = y % 2 == 0;
        let xs: Box<dyn Iterator<Item = u32>> = if ltr {
            Box::new(0..w)
        } else {
            Box::new((0..w).rev())
        };
        for x in xs {
            let [lr, lg, lb] = buf[idx(x, y)];

            // Find nearest palette color. Convert this pixel's linear value
            // back to sRGB just for the OkLab lookup.
            let r_u8 = linear_to_srgb(lr);
            let g_u8 = linear_to_srgb(lg);
            let b_u8 = linear_to_srgb(lb);
            let pal_idx = nearest_oklab(&palette_lab, rgb_to_oklab(r_u8, g_u8, b_u8));

            // Write the sRGB palette color to output.
            let [pr, pg, pb] = palette_rgb[pal_idx];
            out.put_pixel(x, y, image::Rgba([pr, pg, pb, alpha[idx(x, y)]]));

            // Compute error in LINEAR space, propagate in linear space.
            let [plr, plg, plb] = palette_linear[pal_idx];
            let er = lr - plr;
            let eg = lg - plg;
            let eb = lb - plb;

            let mut diffuse = |dx: i32, dy: i32, weight: f32| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                    return;
                }
                let p = &mut buf[idx(nx as u32, ny as u32)];
                p[0] += er * weight;
                p[1] += eg * weight;
                p[2] += eb * weight;
            };

            let sign = if ltr { 1 } else { -1 };
            diffuse(1 * sign, 0, 7.0 / 16.0);
            diffuse(-1 * sign, 1, 3.0 / 16.0);
            diffuse(0, 1, 5.0 / 16.0);
            diffuse(1 * sign, 1, 1.0 / 16.0);
        }
    }
    Ok(out)
}
