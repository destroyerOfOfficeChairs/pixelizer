//! The live, editable representation of a pipeline.
//!
//! This is data-source #1: "the row of operations and their current values."
//! It deliberately does NOT reuse core's typed `Operation` enum as the storage
//! shape. An `Operation` is what core needs at `apply` time; storing it live
//! forces a serde bridge between the generic widgets and the typed fields on
//! every edit. Instead we store a `tag` + a bag of `ParamValue`s — the same
//! shape the descriptor schema (`OP_VARIANTS`) already describes — so a widget
//! reads and writes `values[key]` directly, with no closures and no round-trip.
//!
//! The typed `Operation` is reconstructed once, at the Run boundary
//! (`OpInstance::to_operation`), which is the only place the schema-vs-bag
//! contract is actually checked.

use pixelizer_core::op_schema::{ParamKind, VariantDescriptor, op_variants};
use pixelizer_core::{DitherConfig, Operation};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Value types — one arm per "kind of thing a widget produces". Each arm exists
// because some ParamKind in the schema yields it:
//   Float | Int      -> Num
//   Bool             -> Bool
//   Palette          -> Palette
//   Dither           -> Dither
// ---------------------------------------------------------------------------

/// A leaf value currently being edited. `Num` covers both Float and Int params;
/// the schema's `ParamKind` carries the int-vs-float distinction (range, step,
/// and how to narrow at the boundary), so the value itself needn't re-encode it.
#[derive(Clone, Debug, PartialEq)]
pub enum ParamValue {
    Num(f64),
    Bool(bool),
    Palette(Vec<String>),
    Dither(Option<DitherChoice>),
}

impl ParamValue {
    pub fn as_num(&self) -> Option<f64> {
        match self {
            ParamValue::Num(n) => Some(*n),
            ParamValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ParamValue::Bool(b) => Some(*b),
            ParamValue::Num(n) => Some(*n != 0.0),
            _ => None,
        }
    }
}

/// A chosen dither variant plus current values for that variant's params.
/// Structurally a mini-instance (tag + scalar bag), but its own type: its bag
/// only ever holds Num/Bool, never a nested Palette or Dither. Saying so keeps
/// the recursion provably two levels deep — exactly as deep as the data is.
#[derive(Clone, Debug, PartialEq)]
pub struct DitherChoice {
    pub tag: String,
    pub values: BTreeMap<String, ParamValue>,
}

/// One operation instance in the live pipeline. Pure data: serializable as-is,
/// convertible to core::Operation at the boundary. No UI concerns (see OpRow
/// for the `id` the keyed <For> needs).
#[derive(Clone, Debug, PartialEq)]
pub struct OpInstance {
    pub tag: String,
    pub values: BTreeMap<String, ParamValue>,
}

// ---------------------------------------------------------------------------
// Construction from the schema — the defaults live in OP_VARIANTS, so building
// a fresh instance means reading them, never hard-coding them here.
// ---------------------------------------------------------------------------

/// The default `ParamValue` for one descriptor param, straight from its kind.
fn default_value(kind: ParamKind) -> ParamValue {
    match kind {
        ParamKind::Float { default, .. } => ParamValue::Num(default as f64),
        ParamKind::Int { default, .. } => ParamValue::Num(default as f64),
        ParamKind::Bool { default } => ParamValue::Bool(default),
        ParamKind::Palette { colors } => {
            ParamValue::Palette(colors.iter().map(|s| s.to_string()).collect())
        }
        // A fresh palette-map starts with dithering OFF. `default_tag` is the
        // variant used when the user later switches it on (handled in the UI),
        // not a reason to start in the Some(_) state.
        ParamKind::Dither { .. } => ParamValue::Dither(None),
    }
}

/// Build a fresh, default-valued instance for the op named by `tag`.
/// Returns None for a tag not in OP_VARIANTS.
pub fn default_instance(tag: &str) -> Option<OpInstance> {
    let variant = op_variants().iter().find(|v| v.tag == tag)?;
    let values = variant
        .params
        .iter()
        .map(|p| (p.key.to_string(), default_value(p.kind)))
        .collect();
    Some(OpInstance {
        tag: tag.to_string(),
        values,
    })
}

/// Build a default DitherChoice for a dither variant tag (used when the user
/// switches dithering on or picks a different algorithm). None for unknown tag.
pub fn default_dither_choice(tag: &str) -> Option<DitherChoice> {
    let variant = pixelizer_core::op_schema::dither_variants()
        .iter()
        .find(|v| v.tag == tag)?;
    let values = variant
        .params
        .iter()
        .map(|p| (p.key.to_string(), default_value(p.kind)))
        .collect();
    Some(DitherChoice {
        tag: tag.to_string(),
        values,
    })
}

// ---------------------------------------------------------------------------
// The boundary conversion — the ONE place the typed enum is reconstructed, and
// the ONE place the schema-vs-bag contract is checked. Returns Result so a
// malformed bag (missing key, wrong arm — e.g. from schema/pipeline drift)
// surfaces as a message at Run instead of a panic.
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct BuildError {
    pub op: String,
    pub detail: String,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "couldn't build '{}': {}", self.op, self.detail)
    }
}

impl OpInstance {
    /// Read a Num param and narrow to u32 (for the unsigned-int core fields).
    fn u32_field(&self, key: &str) -> Result<u32, BuildError> {
        let n = self
            .values
            .get(key)
            .and_then(ParamValue::as_num)
            .ok_or_else(|| self.miss(key, "expected a number"))?;
        Ok(n.round().max(0.0) as u32)
    }

    /// Read a Num param as f32 (for the float core fields).
    fn f32_field(&self, key: &str) -> Result<f32, BuildError> {
        let n = self
            .values
            .get(key)
            .and_then(ParamValue::as_num)
            .ok_or_else(|| self.miss(key, "expected a number"))?;
        Ok(n as f32)
    }

    fn miss(&self, key: &str, what: &str) -> BuildError {
        BuildError {
            op: self.tag.clone(),
            detail: format!("param '{key}': {what}"),
        }
    }

    /// Reconstruct the typed core Operation. Called once per op at Run.
    pub fn to_operation(&self) -> Result<Operation, BuildError> {
        Ok(match self.tag.as_str() {
            "downsample" => Operation::Downsample {
                pixel_size: self.u32_field("pixel_size")?,
            },
            "upscale" => Operation::Upscale {
                factor: self.u32_field("factor")?,
            },
            "posterize" => Operation::Posterize {
                levels: self.u32_field("levels")?,
            },
            "blur" => Operation::Blur {
                sigma: self.f32_field("sigma")?,
            },
            "normalize" => Operation::Normalize {
                low: self.f32_field("low")?,
                high: self.f32_field("high")?,
            },
            "palette_map" => {
                let colors = match self.values.get("palette") {
                    Some(ParamValue::Palette(c)) => c.clone(),
                    _ => return Err(self.miss("palette", "expected a palette")),
                };
                let dither = match self.values.get("dither") {
                    Some(ParamValue::Dither(choice)) => {
                        choice.as_ref().map(DitherChoice::to_config).transpose()?
                    }
                    // Absent dither key = no dithering, not an error.
                    None => None,
                    _ => return Err(self.miss("dither", "expected a dither value")),
                };
                Operation::PaletteMap { colors, dither }
            }
            other => {
                return Err(BuildError {
                    op: other.to_string(),
                    detail: "unknown operation tag".to_string(),
                });
            }
        })
    }
}

impl DitherChoice {
    fn f32(&self, key: &str) -> Result<f32, BuildError> {
        self.values
            .get(key)
            .and_then(ParamValue::as_num)
            .map(|n| n as f32)
            .ok_or_else(|| BuildError {
                op: format!("dither:{}", self.tag),
                detail: format!("param '{key}': expected a number"),
            })
    }
    fn bool(&self, key: &str) -> Result<bool, BuildError> {
        self.values
            .get(key)
            .and_then(ParamValue::as_bool)
            .ok_or_else(|| BuildError {
                op: format!("dither:{}", self.tag),
                detail: format!("param '{key}': expected a bool"),
            })
    }

    /// Reconstruct the typed DitherConfig. Hand-written (five arms) so the whole
    /// boundary stays serde-free and uniform with the op conversion above.
    pub fn to_config(&self) -> Result<DitherConfig, BuildError> {
        Ok(match self.tag.as_str() {
            "floyd_steinberg" => DitherConfig::FloydSteinberg {
                bleed: self.f32("bleed")?,
                clamp: self.bool("clamp")?,
            },
            "atkinson" => DitherConfig::Atkinson {
                bleed: self.f32("bleed")?,
                clamp: self.bool("clamp")?,
            },
            "jjn" => DitherConfig::Jjn {
                bleed: self.f32("bleed")?,
                clamp: self.bool("clamp")?,
            },
            "bayer4" => DitherConfig::Bayer4 {
                strength: self.f32("strength")?,
            },
            "bayer8" => DitherConfig::Bayer8 {
                strength: self.f32("strength")?,
            },
            other => {
                return Err(BuildError {
                    op: format!("dither:{other}"),
                    detail: "unknown dither tag".to_string(),
                });
            }
        })
    }
}

// A tiny helper so callers don't reach into op_variants themselves.
#[allow(dead_code)]
pub fn variant_for(tag: &str) -> Option<&'static VariantDescriptor> {
    op_variants().iter().find(|v| v.tag == tag)
}
