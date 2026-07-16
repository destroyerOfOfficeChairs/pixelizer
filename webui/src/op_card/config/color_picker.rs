use super::swatches::PickerAnchor;
use leptos::ev::pointermove;
use leptos::ev::pointerup;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{on_click_outside, use_event_listener};

#[derive(Debug, PartialEq)]
pub enum HexError {
    InvalidLength,
    InvalidCharacter,
}

#[derive(Clone, Copy, PartialEq)]
enum Dragging {
    None,
    Square,
    Slider,
}

#[component]
pub fn ColorPicker(
    anchor: PickerAnchor,
    on_close: Callback<()>,
    on_pick: Callback<String>,
    hex: Signal<String>,
) -> impl IntoView {
    let picker_ref = NodeRef::<leptos::html::Div>::new();
    let input_text = RwSignal::new(hex.get_untracked().trim_start_matches('#').to_string());
    let (h0, s0, v0) = hex_to_rgb(&hex.get_untracked())
        .map(rgb_to_hsv)
        .unwrap_or((0.0, 0.0, 0.0));
    let hue = RwSignal::new(h0);
    let sat = RwSignal::new(s0);
    let val = RwSignal::new(v0);
    let working = Signal::derive(move || rgb_to_hex(hsv_to_rgb((hue.get(), sat.get(), val.get()))));

    let submit_handler = move |_| {
        on_pick.run(working.get());
        leptos::logging::log!("working.get(): {}", working.get());
    };

    let slider: NodeRef<html::Div> = NodeRef::new();
    let square: NodeRef<html::Div> = NodeRef::new();

    let dragging = RwSignal::new(Dragging::None);

    let apply_square = move |client_x: f64, client_y: f64| {
        if let Some(el) = square.get_untracked() {
            let rect = el.get_bounding_client_rect();
            let s = ((client_x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let v = 1.0 - ((client_y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            sat.set(s);
            val.set(v);
        }
    };

    let apply_slider = move |client_x: f64| {
        if let Some(el) = slider.get_untracked() {
            let rect = el.get_bounding_client_rect();
            let frac = ((client_x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            hue.set(frac * 360.0);
        }
    };

    let _ = use_event_listener(window(), pointermove, move |ev| {
        match dragging.get_untracked() {
            Dragging::Square => apply_square(ev.client_x() as f64, ev.client_y() as f64),
            Dragging::Slider => apply_slider(ev.client_x() as f64),
            Dragging::None => {}
        }
    });

    let _ = use_event_listener(window(), pointerup, move |_| {
        dragging.set(Dragging::None);
    });

    let _ = on_click_outside(picker_ref, move |_| on_close.run(()));

    view! {
        <div
            node_ref=picker_ref
            class="fixed z-50 w-60 bg-slate-800 border border-slate-700 rounded-lg shadow-xl \
                   flex flex-col gap-3 p-3 select-none"
            style:left=format!("{}px", anchor.x)
            style:top=format!("{}px", anchor.y)
        >
            // Header: title + close
            <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-slate-400">"Edit color"</span>
                <button
                    class="text-slate-500 hover:text-slate-200 leading-none text-lg"
                    on:click=move |_| on_close.run(())
                >"×"</button>
            </div>

            // S/V square — the hero, given the most space
            <div class="relative w-full h-40 rounded overflow-hidden">
                <div
                    node_ref=square
                    class="w-full h-full cursor-crosshair select-none"
                    style:background=move || format!(
                        "linear-gradient(to bottom, transparent, #000), \
                         linear-gradient(to right, #fff, transparent), \
                         hsl({}, 100%, 50%)",
                        hue.get()
                    )
                    on:pointerdown=move |ev| {
                        ev.prevent_default();
                        dragging.set(Dragging::Square);
                        apply_square(ev.client_x() as f64, ev.client_y() as f64);
                    }
                />
                <div
                    class="absolute w-4 h-4 rounded-full border-2 border-white shadow \
                           pointer-events-none -translate-x-1/2 -translate-y-1/2"
                    style:left=move || format!("{}%", sat.get() * 100.0)
                    style:top=move || format!("{}%", (1.0 - val.get()) * 100.0)
                />
            </div>

            // Hue strip
            <div class="relative w-full h-3">
                <div
                    node_ref=slider
                    class="w-full h-3 rounded-full cursor-pointer select-none"
                    style:background="linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)"
                    on:pointerdown=move |ev| {
                        ev.prevent_default();
                        dragging.set(Dragging::Slider);
                        apply_slider(ev.client_x() as f64);
                    }
                />
                <div
                    class="absolute top-1/2 w-3 h-3 rounded-full bg-white border border-slate-900 \
                           shadow pointer-events-none -translate-x-1/2 -translate-y-1/2"
                    style:left=move || format!("{}%", hue.get() / 360.0 * 100.0)
                />
            </div>

            // Preview pair + hex input
            <div class="flex items-center gap-2">
                // current (was) over new (working), stacked halves in one rounded box
                <div class="flex flex-col w-8 h-8 rounded overflow-hidden border border-slate-700 shrink-0">
                    <div class="flex-1" style:background-color=move || hex.get()/>
                    <div class="flex-1" style:background-color=move || working.get()/>
                </div>
                <div class="flex items-center flex-1 bg-slate-900 border border-slate-700 rounded px-2">
                    <span class="text-slate-500 text-sm font-mono">"#"</span>
                    <input
                        class="flex-1 bg-transparent text-slate-200 text-sm font-mono \
                               px-1 py-1 outline-none w-full"
                        type="text"
                        prop:value=move || input_text.get().trim_start_matches('#').to_string()
                        on:input=move |ev| {
                            let text = event_target_value(&ev);
                            input_text.set(text.clone());
                            if let Ok(rgb) = hex_to_rgb(&text) {
                                let (h, s, v) = rgb_to_hsv(rgb);
                                hue.set(h);
                                sat.set(s);
                                val.set(v);
                            }
                        }
                    />
                </div>
            </div>

            <button
                class="w-full bg-teal-600 hover:bg-teal-500 text-white text-sm font-medium \
                       rounded py-1.5"
                on:click=submit_handler
            >"Done"</button>
        </div>
    }
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), HexError> {
    // Strip leading '#' if present
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    match hex.len() {
        // Handle shorthand 3-character format: "RGB" or "#RGB"
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| HexError::InvalidCharacter)?;
            let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| HexError::InvalidCharacter)?;
            let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| HexError::InvalidCharacter)?;

            // Duplicate the digits (e.g., 'f' becomes 0xff / 255)
            Ok(((r << 4) | r, (g << 4) | g, (b << 4) | b))
        }
        // Handle standard 6-character format: "RRGGBB" or "#RRGGBB"
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| HexError::InvalidCharacter)?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| HexError::InvalidCharacter)?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| HexError::InvalidCharacter)?;

            Ok((r, g, b))
        }
        _ => Err(HexError::InvalidLength),
    }
}

fn rgb_to_hsv(rgb: (u8, u8, u8)) -> (f64, f64, f64) {
    let (r, g, b) = rgb;
    let (r_norm, g_norm, b_norm) = (
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    );
    let min = r_norm.min(g_norm).min(b_norm);
    let max = r_norm.max(g_norm).max(b_norm);
    let delta = max - min;
    let mut hue = 0.0;
    if delta == 0.0 {
        hue = 0.0;
    } else if max == r_norm {
        hue = 60.0 * (((g_norm - b_norm) / delta) % 6.0);
    } else if max == g_norm {
        hue = 60.0 * (((b_norm - r_norm) / delta) + 2.0);
    } else if max == b_norm {
        hue = 60.0 * (((r_norm - g_norm) / delta) + 4.0);
    }
    if hue < 0.0 {
        hue += 360.0;
    }
    let mut sat = 0.0;
    if max == 0.0 {
        sat = 0.0;
    } else if max > 0.0 {
        // sat = (delta / max) * 100.0;
        sat = delta / max;
    }
    // let val = max * 100.0;
    let val = max;
    (hue, sat, val)
}

/// H in [0, 360), S and V in [0, 1]. Returns (r, g, b) as u8.
fn hsv_to_rgb(hsv: (f64, f64, f64)) -> (u8, u8, u8) {
    let (h, s, v) = hsv;

    // Chroma: the "colorfulness" — the spread between the max and min channel.
    let c = v * s;
    // Which 60° sextant of the wheel are we in? (0..6)
    let h_prime = (h % 360.0) / 60.0;
    // Second-largest component, ramping up and down within the sextant.
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    // The amount to lift all three channels so the max lands on v.
    let m = v - c;

    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0), //   0°– 60°  red   → yellow
        1 => (x, c, 0.0), //  60°–120°  yellow→ green
        2 => (0.0, c, x), // 120°–180°  green → cyan
        3 => (0.0, x, c), // 180°–240°  cyan  → blue
        4 => (x, 0.0, c), // 240°–300°  blue  → magenta
        _ => (c, 0.0, x), // 300°–360°  magenta → red
    };

    let to_u8 = |f: f64| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(r1), to_u8(g1), to_u8(b1))
}

fn rgb_to_hex(rgb: (u8, u8, u8)) -> String {
    let (r, g, b) = rgb;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_roundtrip() {
        for hex in [
            "#9bbc0f", "#2c2416", "#ff004d", "#000000", "#ffffff", "#808080",
        ] {
            let rgb = hex_to_rgb(hex).unwrap();
            let back = rgb_to_hex(hsv_to_rgb(rgb_to_hsv(rgb)));
            assert_eq!(hex, back, "roundtrip failed for {hex}");
        }
    }
}
