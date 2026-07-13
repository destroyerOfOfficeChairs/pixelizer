use super::swatches::PickerAnchor;
use leptos::prelude::*;
use leptos_use::on_click_outside;

#[derive(Debug, PartialEq)]
pub enum HexError {
    InvalidLength,
    InvalidCharacter,
}

// #[derive(Debug)]
// pub enum RgbError {
//     InvalidRange,
//     InvalidTupleLength,
// }

#[component]
pub fn ColorPicker(
    anchor: PickerAnchor,
    on_close: Callback<()>,
    on_pick: Callback<String>,
    hex: Signal<String>,
) -> impl IntoView {
    let picker_ref = NodeRef::<leptos::html::Div>::new();
    let working = RwSignal::new(hex.get_untracked());
    let input_text = RwSignal::new(hex.get_untracked().trim_start_matches('#').to_string());

    // let apply = move || {
    //     let text = input_text.get();
    //     if hex_to_rgb(&text).is_ok() {
    //         working.set(format!("#{}", text.trim_start_matches('#')));
    //     }
    // };

    let submit_handler = move |_| {
        on_pick.run(working.get());
        leptos::logging::log!("working.get(): {}", working.get());
    };

    on_click_outside(picker_ref, move |_| on_close.run(()));

    view! {
        <div
            node_ref=picker_ref
            class="fixed z-50 w-48 h-48 bg-slate-800 border border-slate-600 rounded shadow-lg"
            style:left=format!("{}px", anchor.x)
            style:top=format!("{}px", anchor.y)
        >
            <button on:click=move |_| on_close.run(())>"×"</button>
            "picker for swatch " {anchor.sid}
            <div // Swatch being edited
                class="w-8 h-8"
                style:background-color=move || working.get()
            />
            <div // Static swatch
                class="w-8 h-8"
                style:background-color=move || hex.get()
            />
            <input
                type="text"
                prop:value=move || input_text.get()
                on:input=move |ev| {
                    let text = event_target_value(&ev);
                    input_text.set(text.clone());              // buffer always tracks the box
                    if hex_to_rgb(&text).is_ok() {             // gate
                        working.set(format!("#{}", text.trim_start_matches('#')));
                    }
                }
                // on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                //     if ev.key() == "Enter" { apply(); }
                // }
            />
            // <button on:click=move |_| {on_pick.run(working.get()); on_close.run(apply());}>"Submit"</button>
            <button on:click=submit_handler>"Submit"</button>
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
