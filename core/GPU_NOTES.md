# GPU backend notes (prospective)

> **Status: exploration, not implementation.** Nothing described here is built. This file records the analysis behind a possible wgpu rewrite of the core operations so the reasoning doesn't have to be re-derived later. It is not documentation of existing architecture.

## Motivation

The current CPU implementation is correct and not slow enough to *need* the GPU. So a wgpu rewrite isn't a performance necessity — it would be undertaken for its own sake (learning, and headroom for much larger images). That's worth stating plainly, because it changes the calculus: there's no obligation to port everything, and operations that fight the GPU model can be left on the CPU or redesigned without guilt.

## The core finding: operations split by parallelism

The operations fall into two groups based on how well they fit the GPU's data-parallel model, where each output pixel is ideally computed independently and simultaneously.

### Maps cleanly (one independent invocation per output pixel)

- **Pointwise ops** — `posterize`, `palette_map` (flat, no dither), and the planned `contrast`/`brightness`/`saturation`. Each output pixel depends only on its own input pixel. These are the natural starting point: trivial shader logic, all the interesting work is in the host-side pipeline setup.
- **`blur`** — Separable Gaussian is a clean two-pass design (horizontal then vertical), the textbook case for intermediate textures and optionally workgroup shared memory.
- **`kuwahara`** — Fully parallel. Each output pixel reads a fixed window and computes region statistics independently of every other pixel. It's compute-heavy but embarrassingly parallel, so it's a *better* GPU fit than its CPU cost suggests — a point in favor of the GPU direction.
- **`downsample` / `upscale`** — Essentially texture sampling with the right filter mode; nearly free on the GPU.
- **Ordered (Bayer) dithering** — The threshold is a pure function of pixel coordinates, so it parallelizes perfectly. Unlike error diffusion (below), this dithering style ports without compromise.

### Fights the GPU model

- **Error-diffusion dithering** (Floyd–Steinberg, Atkinson, JJN) is inherently *sequential*. Each pixel's output depends on quantization error propagated from already-processed neighbors — a serial dependency chain, the opposite of what the GPU wants. Parallel approximations exist (block-based, or processing along anti-diagonals), but they change the output and add real complexity.
- **`normalize`** needs a whole-image reduction (a histogram, then percentile cutoffs) *before* it can transform any pixel. Reductions are a known GPU pain point, and percentile specifically isn't a simple sum — it's atomics or a multi-pass parallel reduction. Doable, but not trivial.

## The design fork this forces

A rewrite has to make a real decision about the sequential 30%. The two coherent options:

1. **Hybrid — GPU for the parallel ops, CPU for the rest.** Pragmatic and preserves current output exactly. Cost: two code paths to maintain, and a GPU→CPU→GPU round-trip mid-pipeline whenever an error-diffusion or normalize step sits between GPU ops. That readback can erase the GPU's speed advantage depending on pipeline shape.

2. **GPU-only, with a different dithering story.** Drop error diffusion in favor of ordered and blue-noise dithering, both fully parallel. Cleaner architecture and no readbacks, at the cost of Atkinson-style output — which would mean revisiting the earlier "Atkinson empirically looked best" conclusion. `normalize` would still need a real parallel-reduction implementation (or a single readback for just the histogram, which is far cheaper than per-op readbacks).

Neither is strictly better; the choice depends on whether preserving exact current output matters more than architectural cleanliness. Recording both here so the tradeoff is visible when the time comes.

## Open questions to resolve before starting

- Does the target use case (how large do images actually get?) justify the GPU at all, or is this purely for learning?
- If hybrid: which operation orderings are common enough that mid-pipeline readback would dominate, and is that acceptable?
- If GPU-only: is losing error-diffusion dithering an acceptable aesthetic tradeoff, given ordered + blue-noise as replacements?
