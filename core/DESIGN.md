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

## Why the core crate carries UI descriptors

`ui_api.rs` holds descriptor tables: for each operation and each dither variant, the list of its parameters with their names, types, defaults, and valid ranges. This is metadata *about* the config types, sitting in the processing library — which is initially surprising, since core does no rendering and knows nothing about any frontend.

The reason is single-source-of-truth. A frontend building config controls needs to know that `blur` has one `f32` parameter called `sigma` that sensibly ranges roughly 0–10, that `posterize`'s `levels` is an integer with a floor of 2, and so on. Without descriptors, every frontend restates that knowledge in its own UI code, and the two drift: add a parameter to an operation here and the UI silently keeps offering the old set. Putting the descriptors next to the `Operation` and `DitherConfig` enums means the parameter list lives in one place, beside the types it describes, and a frontend derives its controls from that rather than duplicating it.

This is deliberately general — nothing in `ui_api.rs` references Leptos, the DOM, or any specific UI toolkit; the types are plain data. In principle any frontend could consume them. In practice there is one consumer today, the `webui` crate, which renders a generic config card per operation by walking these tables. So treat `ui_api` as "a descriptor API any frontend could use, currently used by webui" rather than a finished public contract — its shape may still move as the one real consumer teaches us what it needs.

Two limits worth recording, both consequences of Rust having no runtime reflection:

- **The tables are hand-maintained, not derived from the enums.** Nothing forces `ui_api` to stay in sync with `Operation`/`DitherConfig`; adding an operation means adding its descriptor row by hand. The descriptor's parameter `key` strings must match the enum's serde field names exactly, because that key is what a frontend uses to read and write the field (see below). A mismatch is a silent bug, not a compile error — the most fragile seam in this design.
- **Serde is the bridge between descriptor values and real config types.** A frontend doesn't hand-write a "tag → variant" constructor. Because `Operation` and `DitherConfig` already derive `Serialize`/`Deserialize` with internal tags (`type` and `algorithm`), a frontend can read a field by serializing an op to a JSON object and reading it by key, and write one back by overwriting that key and deserializing. This reuses the serialization the crate already has rather than maintaining a second, parallel construction path — which is why the descriptor tables only need to carry *metadata*, not logic.

## Operation order matters

The pipeline is just an ordered list, but the order has real consequences:
- `downsample` should generally come before `palette_map`, though you can reverse this order if you like — but it will be slower.
- `normalize` should come before any quantization step (`posterize`, `palette_map`) whose output depends on the input's brightness distribution.
- `upscale` should be the last step.
