//! A drop zone for palette files. Drag a file onto it (or click to browse),
//! it parses the palette and emits `(name, colors)` upward via `on_load`.
//!
//! The component deliberately knows nothing about where palettes are stored —
//! it parses and reports. The parent decides what "adding a palette" means
//! (push into the palette list, auto-select it, commit to the bag, etc.).
//! Same separation as ColorPicker not knowing about `owned`.
//!
//! Formats (per ROADMAP, easiest-first; ASE deliberately skipped):
//!   .hex  — one hex code per line (lospec's most common export)
//!   .gpl  — GIMP palette: header lines, then `R G B  Name` rows
//!   .pal  — JASC palette: "JASC-PAL" / version / count, then `R G B` rows
//!   .txt  — Paint.NET: one AARRGGBB per line, `;` comments
//!
//! All formats normalize to `Vec<String>` of lowercase `#rrggbb` — the same
//! shape `palettes.yaml` produces, so downstream code can't tell the difference.

use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

// ---------------------------------------------------------------------------
// Parsing — pure functions, no DOM, testable with `cargo test`.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum PaletteParseError {
    /// File parsed but produced zero colors.
    Empty,
    /// A line that should have been a color wasn't. (1-based line number.)
    BadLine(usize),
    /// Extension unknown and content didn't match any known format.
    UnknownFormat,
}

impl std::fmt::Display for PaletteParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaletteParseError::Empty => write!(f, "no colors found in file"),
            PaletteParseError::BadLine(n) => write!(f, "couldn't parse line {n}"),
            PaletteParseError::UnknownFormat => write!(f, "unrecognized palette format"),
        }
    }
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Normalize a bare hex string (3 or 6 digits, no '#') to `#rrggbb` lowercase.
fn normalize_hex(h: &str) -> String {
    if h.len() == 3 {
        // Expand shorthand: "9bf" -> "#99bbff"
        let doubled: String = h.chars().flat_map(|c| [c, c]).collect();
        format!("#{}", doubled.to_ascii_lowercase())
    } else {
        format!("#{}", h.to_ascii_lowercase())
    }
}

/// lospec .hex: one color per line, usually bare "9bbc0f", sometimes "#9bbc0f".
/// NOTE: '#' is a color prefix here, NOT a comment marker.
fn parse_hex(text: &str) -> Result<Vec<String>, PaletteParseError> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let h = t.strip_prefix('#').unwrap_or(t);
        let valid = (h.len() == 6 || h.len() == 3) && h.chars().all(|c| c.is_ascii_hexdigit());
        if !valid {
            return Err(PaletteParseError::BadLine(i + 1));
        }
        out.push(normalize_hex(h));
    }
    if out.is_empty() {
        Err(PaletteParseError::Empty)
    } else {
        Ok(out)
    }
}

/// GIMP .gpl: "GIMP Palette" magic, optional "Name:"/"Columns:" headers,
/// '#' comment lines, then `R G B  <name>` rows (name optional).
fn parse_gpl(text: &str) -> Result<Vec<String>, PaletteParseError> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if t.is_empty()
            || t == "GIMP Palette"
            || t.starts_with('#')
            || t.starts_with("Name:")
            || t.starts_with("Columns:")
        {
            continue;
        }
        let mut it = t.split_whitespace();
        let rgb: Option<(u8, u8, u8)> = (|| {
            let r = it.next()?.parse().ok()?;
            let g = it.next()?.parse().ok()?;
            let b = it.next()?.parse().ok()?;
            Some((r, g, b))
        })();
        match rgb {
            Some((r, g, b)) => out.push(rgb_to_hex(r, g, b)),
            None => return Err(PaletteParseError::BadLine(i + 1)),
        }
    }
    if out.is_empty() {
        Err(PaletteParseError::Empty)
    } else {
        Ok(out)
    }
}

/// JASC .pal: "JASC-PAL" magic, a version line ("0100"), a count line,
/// then `R G B` rows.
fn parse_jasc(text: &str) -> Result<Vec<String>, PaletteParseError> {
    let mut lines = text.lines().enumerate();
    match lines.next() {
        Some((_, l)) if l.trim() == "JASC-PAL" => {}
        _ => return Err(PaletteParseError::UnknownFormat),
    }
    // Version and count lines — present in every real file; don't validate
    // the count against the row total, just skip both.
    lines.next();
    lines.next();

    let mut out = Vec::new();
    for (i, line) in lines {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let mut it = t.split_whitespace();
        let rgb: Option<(u8, u8, u8)> = (|| {
            let r = it.next()?.parse().ok()?;
            let g = it.next()?.parse().ok()?;
            let b = it.next()?.parse().ok()?;
            Some((r, g, b))
        })();
        match rgb {
            Some((r, g, b)) => out.push(rgb_to_hex(r, g, b)),
            None => return Err(PaletteParseError::BadLine(i + 1)),
        }
    }
    if out.is_empty() {
        Err(PaletteParseError::Empty)
    } else {
        Ok(out)
    }
}

/// Paint.NET .txt: one AARRGGBB hex per line, ';' starts a comment
/// (whole-line or trailing). Alpha is stripped. 6-digit lines tolerated.
fn parse_paintdotnet(text: &str) -> Result<Vec<String>, PaletteParseError> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        // Everything before the first ';' is the payload.
        let t = line.split(';').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        let all_hex = t.chars().all(|c| c.is_ascii_hexdigit());
        let rgb = match (t.len(), all_hex) {
            (8, true) => &t[2..], // AARRGGBB -> RRGGBB
            (6, true) => t,
            _ => return Err(PaletteParseError::BadLine(i + 1)),
        };
        out.push(normalize_hex(rgb));
    }
    if out.is_empty() {
        Err(PaletteParseError::Empty)
    } else {
        Ok(out)
    }
}

/// Dispatch by extension; fall back to content sniffing for unknown/missing
/// extensions (magic first lines identify GPL and JASC; bare hex is the
/// last resort since it's the least distinctive).
pub fn parse_palette(filename: &str, text: &str) -> Result<Vec<String>, PaletteParseError> {
    let ext = filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("hex") => parse_hex(text),
        Some("gpl") => parse_gpl(text),
        Some("pal") => parse_jasc(text),
        Some("txt") => parse_paintdotnet(text),
        _ => {
            let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            if first.trim() == "GIMP Palette" {
                parse_gpl(text)
            } else if first.trim() == "JASC-PAL" {
                parse_jasc(text)
            } else {
                // Try the two line-of-hex formats; hex first (more common).
                parse_hex(text).or_else(|_| parse_paintdotnet(text))
            }
        }
    }
}

/// The filename without its final extension: "sepia.hex" -> "sepia".
fn file_stem(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| name.to_string())
}

// ---------------------------------------------------------------------------
// The component
// ---------------------------------------------------------------------------

/// Drop zone + click-to-browse for palette files.
///
/// Emits `(name, colors)` on a successful parse: `name` is the filename
/// without extension, `colors` is normalized `#rrggbb` strings. Parse
/// failures are shown inline and nothing is emitted.
#[component]
pub fn PaletteDropZone(on_load: Callback<(String, Vec<String>)>) -> impl IntoView {
    let input_ref: NodeRef<html::Input> = NodeRef::new();
    let drag_over = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    // Shared by the drop handler and the file-input change handler.
    // Captures only Copy handles, so the closure is Copy and both can use it.
    let handle_file = move |file: gloo_file::File| {
        let name = file.name();
        spawn_local(async move {
            match gloo_file::futures::read_as_text(&file).await {
                Ok(text) => match parse_palette(&name, &text) {
                    Ok(colors) => {
                        error.set(None);
                        on_load.run((file_stem(&name), colors));
                    }
                    Err(e) => error.set(Some(e.to_string())),
                },
                Err(_) => error.set(Some("couldn't read file".to_string())),
            }
        });
    };

    view! {
        <div
            class="flex flex-col items-center justify-center gap-1 mx-3 my-2 px-3 py-4 \
                   rounded border border-dashed cursor-pointer select-none \
                   text-slate-500 hover:text-slate-300 hover:border-slate-400 \
                   transition-colors"
            // Highlight while a drag hovers; otherwise the resting border.
            class:border-teal-400=move || drag_over.get()
            class:border-slate-600=move || !drag_over.get()
            on:click=move |_| {
                if let Some(input) = input_ref.get_untracked() {
                    input.click();
                }
            }
            // dragover MUST prevent_default or the browser won't allow a drop.
            on:dragover=move |ev: leptos::ev::DragEvent| {
                ev.prevent_default();
                drag_over.set(true);
            }
            on:dragleave=move |_| drag_over.set(false)
            on:drop=move |ev: leptos::ev::DragEvent| {
                ev.prevent_default();
                drag_over.set(false);
                if let Some(dt) = ev.data_transfer() {
                    if let Some(files) = dt.files() {
                        if let Some(f) = files.get(0) {
                            handle_file(gloo_file::File::from(f));
                        }
                    }
                }
            }
        >
            // pointer-events-none on children: without it, dragging over the
            // text fires dragleave on the zone and the highlight flickers.
            <span class="text-xs pointer-events-none">
                "Drop a palette file — or click to browse"
            </span>
            <span class="text-[10px] text-slate-600 pointer-events-none">
                ".hex · .gpl · .pal · .txt"
            </span>
            {move || error.get().map(|e| view! {
                <span class="text-xs text-red-400 pointer-events-none">{e}</span>
            })}
            <input
                type="file"
                class="hidden"
                accept=".hex,.gpl,.pal,.txt"
                node_ref=input_ref
                // The programmatic input.click() bubbles back up to the zone's
                // on:click, which would call input.click() again. Stop it here.
                on:click=move |ev| ev.stop_propagation()
                on:change=move |ev| {
                    let input: web_sys::HtmlInputElement = event_target(&ev);
                    if let Some(files) = input.files() {
                        if let Some(f) = files.get(0) {
                            handle_file(gloo_file::File::from(f));
                        }
                    }
                    // Reset so re-selecting the SAME file fires change again.
                    input.set_value("");
                }
            />
        </div>
    }
}

// ---------------------------------------------------------------------------
// Tests — parsers are pure, so these run with plain `cargo test`.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_bare_and_prefixed() {
        let text = "9bbc0f\n#2c2416\n\nFA0\n";
        let colors = parse_hex(text).unwrap();
        assert_eq!(colors, vec!["#9bbc0f", "#2c2416", "#ffaa00"]);
    }

    #[test]
    fn hex_rejects_garbage() {
        assert_eq!(parse_hex("notacolor"), Err(PaletteParseError::BadLine(1)));
    }

    #[test]
    fn gpl_skips_headers_and_comments() {
        let text = "GIMP Palette\nName: Test\nColumns: 4\n# a comment\n155 188 15\tLight\n48 98 48 Dark\n";
        let colors = parse_gpl(text).unwrap();
        assert_eq!(colors, vec!["#9bbc0f", "#306230"]);
    }

    #[test]
    fn jasc_skips_magic_version_count() {
        let text = "JASC-PAL\n0100\n2\n255 0 77\n0 0 0\n";
        let colors = parse_jasc(text).unwrap();
        assert_eq!(colors, vec!["#ff004d", "#000000"]);
    }

    #[test]
    fn jasc_wrong_magic_is_unknown() {
        assert_eq!(
            parse_jasc("not-a-pal\n0100\n1\n0 0 0\n"),
            Err(PaletteParseError::UnknownFormat)
        );
    }

    #[test]
    fn paintdotnet_strips_alpha_and_comments() {
        let text = "; Paint.NET Palette File\nFF9BBC0F\nFF2C2416 ; trailing comment\n";
        let colors = parse_paintdotnet(text).unwrap();
        assert_eq!(colors, vec!["#9bbc0f", "#2c2416"]);
    }

    #[test]
    fn sniffing_without_extension() {
        assert!(parse_palette("mystery", "GIMP Palette\n1 2 3\n").is_ok());
        assert!(parse_palette("mystery", "JASC-PAL\n0100\n1\n1 2 3\n").is_ok());
        assert!(parse_palette("mystery", "9bbc0f\n").is_ok());
    }

    #[test]
    fn stem_strips_only_final_extension() {
        assert_eq!(file_stem("sepia.hex"), "sepia");
        assert_eq!(file_stem("my.cool.palette.gpl"), "my.cool.palette");
        assert_eq!(file_stem("noext"), "noext");
    }
}
