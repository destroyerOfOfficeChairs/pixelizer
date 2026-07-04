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
    /// A fixed palette of colors (the palette-map param). `colors` is the
    /// default starting palette; the UI swaps it via the palette picker.
    Palette { colors: [&'static str; 2] },
    /// The palette-map dither param. ...
    Dither { default_tag: &'static str },
}

const DEFAULT_PALETTE: [&str; 2] = ["#000000", "#ffffff"];

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

const PALETTE_MAP_PARAMS: &[ParamDescriptor] = &[
    ParamDescriptor {
        key: "palette",
        label: "palette",
        kind: ParamKind::Palette {
            colors: DEFAULT_PALETTE,
        },
    },
    ParamDescriptor {
        key: "alpha",
        label: "preserve alpha",
        kind: ParamKind::Bool { default: true },
    },
    ParamDescriptor {
        key: "dither",
        label: "dither",
        kind: ParamKind::Dither {
            default_tag: "bayer8",
        },
    },
];

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
    VariantDescriptor {
        tag: "palette_map",
        label: "palette map",
        params: PALETTE_MAP_PARAMS,
    },
];

// A small accessor so the webui doesn't index the slice directly.
pub fn dither_variants() -> &'static [VariantDescriptor] {
    DITHER_VARIANTS
}

pub fn op_variants() -> &'static [VariantDescriptor] {
    OP_VARIANTS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `ParamKind::Dither { default_tag }` must name a real variant in
    /// DITHER_VARIANTS. This is the one string that could silently drift from
    /// the table now that the nested-enum taxonomy is gone; the test ties it
    /// back so a typo fails `cargo test` instead of failing at runtime.
    #[test]
    fn dither_default_tags_exist() {
        let known = |tag: &str| DITHER_VARIANTS.iter().any(|v| v.tag == tag);
        for variant in OP_VARIANTS {
            for p in variant.params {
                if let ParamKind::Dither { default_tag } = p.kind {
                    assert!(
                        known(default_tag),
                        "op '{}' param '{}' has default_tag '{}' \
                         with no matching DITHER_VARIANTS entry",
                        variant.tag,
                        p.key,
                        default_tag,
                    );
                }
            }
        }
    }
}

/// The UI label for an op named by its tag. Falls back to the tag itself if
/// it isn't in OP_VARIANTS (shouldn't happen for a live instance).
pub fn label_for_tag(tag: &str) -> &'static str {
    OP_VARIANTS
        .iter()
        .find(|v| v.tag == tag)
        .map(|v| v.label)
        .unwrap_or("unknown")
}

/// Every op the "add operation" menu should offer, as (tag, label) pairs.
/// Straight from the table — no separate list to maintain.
pub fn all_op_menu() -> Vec<(&'static str, &'static str)> {
    OP_VARIANTS.iter().map(|v| (v.tag, v.label)).collect()
}
