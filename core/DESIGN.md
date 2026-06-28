# Design and implementation notes

Rationale for the architectural and algorithmic choices in `pixelizer-core`. For the user-facing operations reference, see [README.md](README.md). For not-yet-built operations and features, see [ROADMAP.md](ROADMAP.md). For a prospective GPU backend, see [GPU_NOTES.md](GPU_NOTES.md).

## Why perceptual color matching?

`palette_map` uses OkLab distance rather than RGB distance to decide which palette color is "nearest" to each pixel. OkLab is a perceptually uniform color space — equal numeric distances correspond to equal perceived color differences. In RGB, the difference between two greens can numerically equal the difference between a green and a brown, even though the second pair looks more different to a human. OkLab fixes this.

## Why error diffusion happens in linear-light RGB

sRGB values stored in image files are gamma-encoded — they're nonlinear with respect to actual light intensity. Adding 0.1 to an sRGB value doesn't add a consistent amount of light depending on where you start.

When dithering propagates quantization error to neighboring pixels, that error needs to be arithmetic on light intensities, not on gamma-encoded numbers. Otherwise, the algorithm generates the wrong corrections and produces too-dark midtones and color casts. `palette_map_diffuse` converts to linear-light floats, dithers in that space, and converts back to sRGB only when writing each output pixel.

## Averaging in a linear space

The same gamma concern applies to any operation that averages pixels, not just dithering. Averaging gamma-encoded sRGB values darkens the result, because the midpoint of two encoded values is not the encoding of the midpoint of their intensities. `blur` therefore averages in a linear space and converts back only when writing output. Any future operation that combines pixels (e.g. the planned `kuwahara` and `sharpen`, see ROADMAP.md) should do the same.

## Why the palette is stored three ways

`prepare_palette` returns palette data in three representations:
- `rgb` — Original sRGB bytes, used for writing output pixels.
- `lab` — OkLab values, used for nearest-color decisions.
- `linear` — Linear-light floats, used for error propagation during dithering.

Each representation serves a different purpose. We compute them once during palette setup and pass references to them through the inner loops.

## Operation order matters

The pipeline is just an ordered list, but the order has real consequences:
- `downsample` should generally come before `palette_map`, though you can reverse this order if you like — but it will be slower.
- `normalize` should come before any quantization step (`posterize`, `palette_map`) whose output depends on the input's brightness distribution.
- `upscale` should be the last step.
