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

```yaml
operations:
  - type: downsample
    pixel_size: 16
    trim: trim_all
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
- `pixel_size: u32`
  - Sets the "pixel size".
- `trim: trim_top | trim_bottom | trim_left | trim_right | trim_vertical | trim_horizontal | trim_top_and_left | trim_top_and_right | trim_bottom_and_left | trim_bottom_and_right | trim_all | trim_none`
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

## Design notes

### Why perceptual color matching?

`palette_map` uses OkLab distance rather than RGB distance to decide which palette color is "nearest" to each pixel. OkLab is a perceptually uniform color space — equal numeric distances correspond to equal perceived color differences. In RGB, the difference between two greens can numerically equal the difference between a green and a brown, even though the second pair looks more different to a human. OkLab fixes this.

### Why error diffusion happens in linear-light RGB

sRGB values stored in image files are gamma-encoded — they're nonlinear with respect to actual light intensity. Adding 0.1 to an sRGB value doesn't add a consistent amount of light depending on where you start.

When dithering propagates quantization error to neighboring pixels, that error needs to be arithmetic on light intensities, not on gamma-encoded numbers. Otherwise, the algorithm generates the wrong corrections and produces too-dark midtones and color casts. `palette_map_diffuse` converts to linear-light floats, dithers in that space, and converts back to sRGB only when writing each output pixel.

### Why the palette is stored three ways

`prepare_palette` returns palette data in three representations:
- `rgb` — Original sRGB bytes, used for writing output pixels.
- `lab` — OkLab values, used for nearest-color decisions.
- `linear` — Linear-light floats, used for error propagation during dithering.

Each representation serves a different purpose. We compute them once during palette setup and pass references to them through the inner loops.

### Operation order matters

The pipeline is just an ordered list, but the order has real consequences:
- `trim_*` should come before `downsample` (it prepares the image dimensions).
- `downsample` should generally come before `palette_map` if you want pixel-art-resolution dithering; after if you want full-resolution dithering with nearest-neighbor downscaling.
- `normalize` should come before any quantization step (`posterize`, `palette_map`) whose output depends on the input's brightness distribution.

## Module layout

- `lib.rs` — Pipeline definition, `Operation` enum, `apply` orchestrator.
- `color_utils.rs` — OkLab conversion, palette preparation, hex parsing.
- `palette_map.rs` — Three palette-mapping algorithms (flat, error-diffusion, ordered).
- `posterize.rs`, `blur.rs`, `normalize.rs`, `downsample.rs`, `upscale.rs`, `trim_height.rs`, `trim_width.rs` — One per pipeline operation.

## References

- Tanner Helland, ["Image Dithering: Eleven Algorithms and Source Code
"](https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html)
- Björn Ottosson, ["A perceptual color space for image processing"](https://bottosson.github.io/posts/oklab/)
