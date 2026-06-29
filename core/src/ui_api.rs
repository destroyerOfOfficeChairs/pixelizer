// ----------------------------------------------------------------------------
// 1. DESCRIPTOR TYPES  —  core (e.g. core/src/describe.rs)
// ----------------------------------------------------------------------------
// These are deliberately small and UI-agnostic. They describe a parameter
// well enough that a generic widget can render it, but they name nothing
// Leptos- or DOM-specific. Core stays free of frontend concepts.

/// One tunable parameter of an operation or dither variant.
#[derive(Clone, Copy, Debug)]
pub struct ParamDescriptor {
    /// The struct field name, and the serde key. e.g. "bleed".
    pub key: &'static str,
    /// Human-facing label for the UI. e.g. "Error bleed".
    pub label: &'static str,
    pub kind: ParamKind,
}

/// The type of a parameter, carrying the metadata a widget needs to render it.
/// This is where the range/default info lives — the stuff a serde schema
/// *can't* express because it isn't in the type.
#[derive(Clone, Copy, Debug)]
pub enum ParamKind {
    /// A float, rendered as a slider (your NumberSlider).
    Float {
        default: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    /// An integer, rendered as a slider or stepper.
    Int { default: i64, min: i64, max: i64 },
    /// A bool, rendered as a checkbox/toggle.
    Bool { default: bool },
}

/// One selectable variant (e.g. one dither algorithm), with its parameters.
#[derive(Clone, Debug)]
pub struct VariantDescriptor {
    /// The serde tag value. e.g. "atkinson" — what goes in `algorithm:`.
    pub tag: &'static str,
    /// Human-facing name for the dropdown. e.g. "Atkinson".
    pub label: &'static str,
    /// The parameters this variant exposes.
    pub params: &'static [ParamDescriptor],
}

// ----------------------------------------------------------------------------
// 2. THE DITHER DESCRIPTOR TABLE  —  core (next to DitherConfig)
// ----------------------------------------------------------------------------
// This is the single source of truth for the UI. Note it sits right next to
// the enum, so when you add a variant the table is staring at you.
//
// The three diffusion variants share the same two params, so we name that
// shared slice once and reuse it — adding a fourth diffusion kernel is then
// a one-line table row.

const DIFFUSE_PARAMS: &[ParamDescriptor] = &[
    ParamDescriptor {
        key: "bleed",
        label: "Error bleed",
        kind: ParamKind::Float {
            default: 1.0,
            min: 0.0,
            max: 1.0,
            step: 0.05,
        },
    },
    ParamDescriptor {
        key: "clamp",
        label: "Clamp to palette range",
        kind: ParamKind::Bool { default: false },
    },
];

const BAYER_PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
    key: "strength",
    label: "Dither strength",
    kind: ParamKind::Float {
        default: 32.0,
        min: 0.0,
        max: 64.0,
        step: 1.0,
    },
}];

const DOWNSAMPLE_PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
    key: "pixel_size",
    label: "pixel size",
    kind: ParamKind::Int {
        default: 8,
        min: 1,
        max: 64,
    },
}];

const UPSCALE_PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
    key: "factor",
    label: "scale factor",
    kind: ParamKind::Int {
        default: 8,
        min: 1,
        max: 64,
    },
}];

const NORMALIZE_PARAMS: &[ParamDescriptor] = &[
    ParamDescriptor {
        key: "low",
        label: "low value cutoff",
        kind: ParamKind::Float {
            default: 0.01,
            min: 0.0,
            max: 1.0,
            step: 0.01,
        },
    },
    ParamDescriptor {
        key: "high",
        label: "high value cutoff",
        kind: ParamKind::Float {
            default: 0.99,
            min: 0.0,
            max: 1.0,
            step: 0.01,
        },
    },
];

const BLUR_PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
    key: "sigma",
    label: "sigma",
    kind: ParamKind::Float {
        default: 1.0,
        min: 0.0,
        max: 10.0,
        step: 0.1,
    },
}];

const POSTERIZE_PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
    key: "levels",
    label: "levels",
    kind: ParamKind::Int {
        default: 4,
        min: 2,
        max: 16,
    },
}];

/// Every dither variant the UI should offer. The `None` (no dithering) case
/// is handled by the UI separately — this table is the `Some(_)` options.
pub const DITHER_VARIANTS: &[VariantDescriptor] = &[
    VariantDescriptor {
        tag: "floyd_steinberg",
        label: "Floyd–Steinberg",
        params: DIFFUSE_PARAMS,
    },
    VariantDescriptor {
        tag: "atkinson",
        label: "Atkinson",
        params: DIFFUSE_PARAMS,
    },
    VariantDescriptor {
        tag: "jjn",
        label: "JJN",
        params: DIFFUSE_PARAMS,
    },
    VariantDescriptor {
        tag: "bayer4",
        label: "Bayer 4×4",
        params: BAYER_PARAMS,
    },
    VariantDescriptor {
        tag: "bayer8",
        label: "Bayer 8×8",
        params: BAYER_PARAMS,
    },
];

pub const OP_VARIANTS: &[VariantDescriptor] = &[
    VariantDescriptor {
        tag: "downsample",
        label: "downsample",
        params: DOWNSAMPLE_PARAMS,
    },
    VariantDescriptor {
        tag: "upscale",
        label: "upscale",
        params: UPSCALE_PARAMS,
    },
    VariantDescriptor {
        tag: "normalize",
        label: "normalize",
        params: NORMALIZE_PARAMS,
    },
    VariantDescriptor {
        tag: "blur",
        label: "blur",
        params: BLUR_PARAMS,
    },
    VariantDescriptor {
        tag: "posterize",
        label: "posterize",
        params: POSTERIZE_PARAMS,
    },
];

// A small accessor so the webui doesn't index the slice directly.
pub fn dither_variants() -> &'static [VariantDescriptor] {
    DITHER_VARIANTS
}

pub fn op_variants() -> &'static [VariantDescriptor] {
    OP_VARIANTS
}
