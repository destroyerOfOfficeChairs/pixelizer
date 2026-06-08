use pixelizer_core::{Operation, Pipeline, TrimMode, apply};

fn make_test_image(width: u32, height: u32) -> pixelizer_core::Image {
    let mut img = pixelizer_core::Image::new(width, height);
    for y in 0..height {
        for x in 0..width {
            // Gradient: x affects red, y affects blue.
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let b = ((y as f32 / height as f32) * 255.0) as u8;
            img.put_pixel(x, y, image::Rgba([r, 128, b, 255]));
        }
    }
    img
}

#[test]
fn empty_pipeline_is_identity() {
    let img = make_test_image(10, 10);
    let original = img.clone();
    let pipeline = Pipeline { operations: vec![] };
    let out = apply(&pipeline, img).unwrap();
    assert_eq!(out.dimensions(), original.dimensions());
    assert_eq!(out.get_pixel(5, 5), original.get_pixel(5, 5));
}

#[test]
fn pixel_size_must_be_first() {
    let img = make_test_image(10, 10);
    let pipeline = Pipeline {
        operations: vec![Operation::Downsample, Operation::PixelSize { size: 2 }],
    };
    let result = apply(&pipeline, img);
    assert!(matches!(
        result,
        Err(pixelizer_core::PixelizerError::OrderError(_))
    ));
}

#[test]
fn trim_and_downsample_produces_expected_size() {
    // 13x17 → trim to 12x16 (divisible by 4) → downsample by 4 → 3x4
    let img = make_test_image(13, 17);
    let pipeline = Pipeline {
        operations: vec![
            Operation::PixelSize { size: 4 },
            Operation::TrimWidth {
                mode: TrimMode::Both,
            },
            Operation::TrimHeight {
                mode: TrimMode::Both,
            },
            Operation::Downsample,
        ],
    };
    let out = apply(&pipeline, img).unwrap();
    assert_eq!(out.dimensions(), (3, 4));
}

#[test]
fn upscale_doubles_dimensions() {
    let img = make_test_image(5, 7);
    let pipeline = Pipeline {
        operations: vec![Operation::Upscale { factor: 2 }],
    };
    let out = apply(&pipeline, img).unwrap();
    assert_eq!(out.dimensions(), (10, 14));
}

#[test]
fn palette_map_outputs_only_palette_colors() {
    use std::collections::HashSet;

    let img = make_test_image(20, 20);
    let pipeline = Pipeline {
        operations: vec![Operation::PaletteMap {
            colors: vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()],
            dither: None,
        }],
    };
    let out = apply(&pipeline, img).unwrap();

    let expected: HashSet<[u8; 3]> = [[255, 0, 0], [0, 255, 0], [0, 0, 255]]
        .into_iter()
        .collect();

    for pixel in out.pixels() {
        let rgb = [pixel.0[0], pixel.0[1], pixel.0[2]];
        assert!(expected.contains(&rgb), "unexpected color {:?}", rgb);
    }
}

#[test]
fn yaml_roundtrip() {
    let yaml = r##"
operations:
  - type: pixel_size
    size: 8
  - type: posterize
    levels: 4
  - type: palette_map
    colors:
      - "#000000"
      - "#ffffff"
    dither:
      algorithm: floyd_steinberg
      bleed: 0.5
      clamp: true
"##;
    let pipeline: Pipeline = serde_yaml::from_str(yaml).expect("should parse");
    assert_eq!(pipeline.operations.len(), 3);
}
