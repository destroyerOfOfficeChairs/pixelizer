use crate::Image;

pub fn normalize(image: Image, low_percentile: f32, high_percentile: f32) -> Image {
    let (w, h) = image.dimensions();
    let total_pixels = (w * h) as usize;

    // Collect each channel's values into sortable vectors.
    let mut channels: [Vec<u8>; 3] = [
        Vec::with_capacity(total_pixels),
        Vec::with_capacity(total_pixels),
        Vec::with_capacity(total_pixels),
    ];
    for pixel in image.pixels() {
        channels[0].push(pixel.0[0]);
        channels[1].push(pixel.0[1]);
        channels[2].push(pixel.0[2]);
    }
    for ch in &mut channels {
        ch.sort_unstable();
    }

    // Find the percentile cutoffs per channel.
    let low_idx = ((total_pixels as f32) * low_percentile).floor() as usize;
    let high_idx =
        (((total_pixels as f32) * high_percentile).ceil() as usize).min(total_pixels - 1);

    let mut lows = [0u8; 3];
    let mut highs = [255u8; 3];
    for c in 0..3 {
        lows[c] = channels[c][low_idx];
        highs[c] = channels[c][high_idx];
    }

    // Stretch each channel from [low, high] to [0, 255].
    let mut out = Image::new(w, h);
    for (x, y, pixel) in image.enumerate_pixels() {
        let mut new_channels = [0u8; 4];
        for c in 0..3 {
            if highs[c] == lows[c] {
                new_channels[c] = pixel.0[c];
            } else {
                let range = (highs[c] - lows[c]) as f32;
                // Values below `lows[c]` clamp to 0; above `highs[c]` clamp to 255.
                let shifted = (pixel.0[c] as f32 - lows[c] as f32).max(0.0);
                new_channels[c] = ((shifted / range) * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
        new_channels[3] = pixel.0[3];
        out.put_pixel(x, y, image::Rgba(new_channels));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_stretches_dim_image_to_full_range() {
        // An image where all pixels are in [100, 150] should stretch to [0, 255].
        let mut img = Image::new(3, 1);
        img.put_pixel(0, 0, image::Rgba([100, 100, 100, 255]));
        img.put_pixel(1, 0, image::Rgba([125, 125, 125, 255]));
        img.put_pixel(2, 0, image::Rgba([150, 150, 150, 255]));

        let out = normalize(img, 0.0, 1.0);

        // With 0/1.0 percentiles, exact min/max should stretch perfectly.
        assert_eq!(out.get_pixel(0, 0).0[0], 0);
        assert_eq!(out.get_pixel(2, 0).0[0], 255);
        // Middle should be roughly halfway.
        let mid = out.get_pixel(1, 0).0[0];
        assert!(mid > 120 && mid < 135, "got {}", mid);
    }

    #[test]
    fn normalize_preserves_alpha() {
        let mut img = Image::new(2, 1);
        img.put_pixel(0, 0, image::Rgba([100, 100, 100, 50]));
        img.put_pixel(1, 0, image::Rgba([200, 200, 200, 200]));
        let out = normalize(img, 0.0, 1.0);
        assert_eq!(out.get_pixel(0, 0).0[3], 50);
        assert_eq!(out.get_pixel(1, 0).0[3], 200);
    }

    #[test]
    fn normalize_handles_uniform_image() {
        // All pixels identical → should not divide by zero, just pass through.
        let mut img = Image::new(3, 1);
        for x in 0..3 {
            img.put_pixel(x, 0, image::Rgba([128, 128, 128, 255]));
        }
        let out = normalize(img, 0.0, 1.0);
        for x in 0..3 {
            assert_eq!(out.get_pixel(x, 0).0[0], 128);
        }
    }
}
