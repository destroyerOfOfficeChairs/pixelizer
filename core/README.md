# pixelizer-core

A pipelined image-processing library for transforming photographs into pixel art.
Operations are described as a list of steps in YAML (or any serde format) and applied in sequence to produce a final image.

## Quickstart

```rust
use pixelizer_core::{apply, Pipeline};

let yaml = std::fs::read_to_string("pipeline.yaml")?;
let pipeline: Pipeline = serde_yaml::from_str(&yaml)?;
let image = image::open("input.png")?.to_rgba8();
let output = apply(&pipeline, image)?;
output.save("output.png")?;
```

## Pipeline format

A pipeline is a list of operations. Each operation has a `type` field identifying it and optional parameters. Operations are applied top-to-bottom.

### Example yaml:

```yaml
operations:
  - type: downsample
    pixel_size: 16
  - type: posterize
    levels: 5
  - type: blur
    sigma: 4.0
  - type: normalize
    low: 0.01
    high: 0.99
  - type: palette_map
    colors:
      - "#2c2416"
      - "#6e5a3d"
      - "#a08658"
      - "#c4a875"
    dither:
      algorithm: atkinson
      bleed: 0.2
      clamp: false
  - type: upscale
    factor: 16
```

### Operations

**`downsample`** — Nearest-neighbor downscale by `pixel_size`. After trimming, the output dimensions are evenly divisible.
- `pixel_size: u32`: Sets the pixel size.
- Crops the image so dimensions are evenly divisible by `pixel_size`. This avoids fractional pixels when downsampling.

**`upscale`** — Nearest-neighbor upscale by an integer factor. Used at the end of a pipeline to make output pixel art viewable at sensible sizes.
- `factor: u32`

**`posterize`** — Reduces each color channel to N evenly-spaced levels. Produces classic banded color regions. `levels: 4` gives 64 total colors.
- `levels: u32` (minimum 2)

**`blur`** — Gaussian blur. Smooths the input so adjacent similar pixels collapse together when quantized.
- `sigma: f32`

**`normalize`** — Stretches each channel so a chosen percentile of pixels fills the 0–255 range. Useful when the image's brightness distribution doesn't match the palette's.
- `low: f32` — Percentile cutoff for the dark end (default 0.01)
- `high: f32` — Percentile cutoff for the bright end (default 0.99)

**`palette_map`** — Maps each pixel to its perceptually-nearest color in a user-specified palette, using OkLab distance.
- `colors: Vec<String>` — Hex color strings, e.g. `"#ff0000"`
- `dither` (optional) — One of:
  - `algorithm: floyd_steinberg | atkinson | jjn` plus:
    - `bleed: f32` — Fraction of quantization error to propagate (default 1.0). Lower values reduce bleeding for palettes that can't represent the input's brightness range.
    - `clamp: bool` — Constrain the error-diffusion buffer to the palette's range. Helps when the palette can't represent brights or darks (default false).
  - `algorithm: bayer4 | bayer8` plus:
    - `strength: f32` — Magnitude of the per-pixel dither bias (default 32.0).

## Module layout

- `lib.rs` — Pipeline definition, `Operation` enum, `apply` orchestrator.
- `color_utils.rs` — OkLab conversion, palette preparation, hex parsing.
- `palette_map.rs` — Three palette-mapping algorithms (flat, error-diffusion, ordered).
- `posterize.rs`, `blur.rs`, `normalize.rs`, `downsample.rs`, `upscale.rs` — One per pipeline operation.

For the rationale behind these design choices — perceptual color matching, linear-light error diffusion, operation ordering, and more — see [DESIGN.md](DESIGN.md). For notes on a possible GPU backend, see [GPU_NOTES.md](GPU_NOTES.md).

## References

- Tanner Helland, ["Image Dithering: Eleven Algorithms and Source Code"](https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html)
- Björn Ottosson, ["A perceptual color space for image processing"](https://bottosson.github.io/posts/oklab/)
