//! String-presentation helpers keyed by tag. Small now; likely home for
//! tooltips, help text, or i18n later.

use super::OP_VARIANTS;

/// The UI label for an op named by its tag. Falls back to "unknown" if the
/// tag isn't in OP_VARIANTS (shouldn't happen for a live instance).
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
