# Roadmap

Operations and features that are **not yet built**. This file exists so the design thinking behind each idea survives a long gap away from the project — each entry records not just *what* but *why* and a rough *how*, enough to start from rather than re-deriving from scratch.

For the rationale behind code that already exists, see [DESIGN.md](DESIGN.md). For a separate, larger exploration of a possible GPU backend, see [GPU_NOTES.md](GPU_NOTES.md).

---

## Easy Win

Make it so that image file sizes are super small. Take advantage of the fact that output images will have a limited color palette.

## Operations

### Resize

Currently, the only way to pixelize an image is to specify the desired pixel size.

Create an operation that allows the user to specify the desired output size of a pixelized image.

Make controls to allow the user to preserve aspect ratio. Also, consider allowing the user to select which filtering type they want.

### `kuwahara` — edge-preserving smoothing

**Why.** Does what `blur` can't: smooth *within* regions without smoothing *across* edges. A Gaussian softens everything uniformly, including the boundaries worth keeping sharp before quantization. Kuwahara flattens flat areas into solid color while leaving edges intact — exactly the input quantization wants, since it collapses noise and gradients into the uniform patches a small palette represents cleanly. It's arguably a better aesthetic fit for this pipeline than any other planned op.

**How.** For each pixel, consider several overlapping sub-regions of the surrounding window, compute each region's variance, and output the *mean* of the lowest-variance region. Low-variance regions are the ones not straddling an edge, so the output is drawn from whichever neighborhood is most internally uniform — which is why edges survive.

Design decisions to make:
- **Variance on a single channel, not per-channel.** Per-channel variance gives three different "winning" regions with no coherent way to combine them. Compute variance on OkLab L (perceptual lightness) to pick the region, then copy that region's mean *color*.
- **Means computed in a linear space** (linear-light RGB or OkLab), not gamma-encoded sRGB — see "Averaging in a linear space" in DESIGN.md. Averaging sRGB darkens the flat regions.
- **Square-quadrant variant first.** Simplest and most instructive, but has visible blocky artifacts on close inspection. Generalized Kuwahara (smooth weighting over more sectors) and anisotropic Kuwahara (sectors following local structure) look substantially better at substantially more cost — future refinements, not the first pass.

**Parameter.** `radius: u32` — half-size of the sampling window.

**Cost.** Heavier than the separable Gaussian — each output pixel computes mean and variance over multiple overlapping regions. Matters most in the webui (synchronous, main-thread): a strong candidate for an "expensive op" warning. Note it's fully parallel, so it ports cleanly to the GPU (see GPU_NOTES.md).

**Ordering.** Before quantization (its whole purpose), and usually before `downsample` so it works on full-resolution detail.

### `contrast` / `brightness`

**Why.** `normalize` stretches the brightness distribution automatically; explicit contrast/brightness curves give manual control over how the input tone maps onto a palette, which `normalize` alone can't express. Cheap and frequently wanted.

**How.** Pointwise tone curve. Apply in linear-light to avoid the gamma artifacts that plague sRGB-space arithmetic; gamma/contrast applied directly to sRGB values shifts midtones in ways that look wrong.

**Parameters.** `amount: f32` (or separate `contrast`/`brightness` scalars).

**Ordering.** Before quantization, like `normalize`.

### `saturation`

**Why.** Perceptual nearest-color matching in `palette_map` tends to *mute* an image — boosting saturation beforehand often produces punchier pixel art. Cheap, and composes naturally with the existing OkLab machinery.

**How.** Work in OkLab and scale the a/b chroma channels. Pointwise, so trivially parallel.

**Parameter.** `amount: f32` — chroma multiplier.

**Ordering.** Before `palette_map`.

### `hue_rotate`

**Why.** Cheap in OkLab, and opens up palette-shifting / color-grading effects. Low priority but nearly free once the OkLab pointwise plumbing for `saturation` exists.

**How.** Rotate the a/b chroma vector by a fixed angle in OkLab. Pointwise.

**Parameter.** `degrees: f32`.

### `sharpen`

**Why.** A natural counterpart to the existing `blur`, useful for recovering definition lost to `downsample`.

**How.** Unsharp mask: blur the image, then add back a scaled difference between the original and the blurred version. Reuses the existing blur path. Average in linear space, same as blur.

**Parameters.** `amount: f32`, and a `sigma: f32` for the underlying blur.

### `edge_detect` / `outline`

**Why.** Pixel art frequently benefits from darkened outlines around regions. Detected edges can be composited as the darkest palette color after `palette_map`.

**How.** Sobel, or difference-of-gaussians, to produce an edge map. Open question: whether this is one op that outputs an edge mask, or a combined op that detects edges and composites them onto the image. The compositing-onto-palette behavior is the genuinely useful end goal but couples it to `palette_map`; worth thinking through before implementing.

**Parameters.** TBD — likely a threshold and an output/blend mode.

### Adaptive palette generation — `quantize_kmeans` / `quantize_median_cut`

**Why.** *The highest-value item here.* Today `palette_map` requires a hand-specified palette; this generates an N-color palette *from the image*, making the tool useful without curating palettes by hand. Reuses the entire existing OkLab + dithering path — only the palette-selection front-end is new.

**How.** Two approaches, both clustering in OkLab so the resulting palette is perceptually sensible:
- **k-means** — higher quality, iterative, slower. Cluster pixel colors in OkLab; the cluster centroids become the palette.
- **median cut** — cheaper, non-iterative adaptive alternative. Good fallback / fast path.

Open question: is this a *new operation* that emits a palette into a following `palette_map`, or a *mode* of `palette_map` where `colors` is replaced by a "generate N from image" option? The latter is probably the cleaner UX, but the operation pipeline passes images, not palettes, between steps — so generating and consuming the palette likely has to happen inside a single op.

**Parameters.** `n: u32` (color count), plus algorithm selection.

### `crop`

**Why.** `downsample`'s trimming handles divisibility, but deliberate compositional cropping is a separate need not currently expressible.

**How.** Straightforward rectangular crop.

**Parameters.** `x`, `y`, `width`, `height` (all `u32`).

---

## Suggested priority

Rough ordering by value-to-effort, for picking the project back up:

1. **Adaptive palette generation** — removes the biggest friction point (palette curation), highest payoff, reuses existing infrastructure.
2. **`kuwahara`** — best aesthetic fit for the pipeline's purpose.
3. **`saturation` + `contrast`** — cheap, high-frequency tone control; `saturation` builds the OkLab pointwise plumbing that `hue_rotate` then reuses for free.
4. **`sharpen`** — cheap, reuses the blur path.
5. **`edge_detect` / `outline`** — useful but needs a design decision about coupling to `palette_map`.
6. **`crop`, `hue_rotate`** — nice-to-haves, low urgency.

## Larger explorations

- **GPU backend** — a possible wgpu rewrite of the core operations. Self-contained analysis with its own design fork; see [GPU_NOTES.md](GPU_NOTES.md).
