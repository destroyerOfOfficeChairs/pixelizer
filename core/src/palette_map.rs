use crate::Image;
use crate::PixelizerError::HexParseError;
use crate::PixelizerError::NoColorsError;

#[derive(Clone, Copy)]
struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

fn parse_hex(s: &str) -> Result<[u8; 3], crate::PixelizerError> {
    let s = s.strip_prefix('#').unwrap_or(s);

    if s.len() != 6 {
        return Err(HexParseError("This is not a hex color.".to_owned()));
    }

    let r = u8::from_str_radix(&s[0..2], 16)
        .map_err(|_| HexParseError("Red is malformed.".to_owned()))?;

    let g = u8::from_str_radix(&s[2..4], 16)
        .map_err(|_| HexParseError("Green is malformed.".to_owned()))?;

    let b = u8::from_str_radix(&s[4..6], 16)
        .map_err(|_| HexParseError("Blue is malformed.".to_owned()))?;

    Ok([r, g, b])
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_oklab(r: u8, g: u8, b: u8) -> Oklab {
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    // Linear RGB -> LMS
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    // Nonlinearity
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS' -> OkLab
    Oklab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

pub fn palette_map(image: Image, colors: &[String]) -> Result<Image, crate::PixelizerError> {
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

    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let idx = nearest_oklab(&palette_lab, rgb_to_oklab(r, g, b));
        let [pr, pg, pb] = palette_rgb[idx];
        out.put_pixel(x, y, image::Rgba([pr, pg, pb, a]));
    }
    Ok(out)
}

fn nearest_oklab(palette: &[Oklab], target: Oklab) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by(|(_, x), (_, y)| {
            let dx = (x.l - target.l).powi(2) + (x.a - target.a).powi(2) + (x.b - target.b).powi(2);
            let dy = (y.l - target.l).powi(2) + (y.a - target.a).powi(2) + (y.b - target.b).powi(2);
            dx.partial_cmp(&dy).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap()
}
