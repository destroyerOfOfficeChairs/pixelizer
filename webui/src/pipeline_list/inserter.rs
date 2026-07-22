use leptos::prelude::*;

#[component]
pub fn Inserter(
    /// Whether this inserter should stay permanently visible and expanded.
    #[prop(into, optional)]
    always_expanded: Signal<bool>,
) -> impl IntoView {
    let expanding_line_class = move || {
        let base = "absolute w-full h-[1px] bg-teal-400 transition-transform\
            duration-300 origin-center pointer-events-none";
        if always_expanded.get() {
            format!("{} scale-x-100", base)
        } else {
            format!("{} scale-x-50 group-hover:scale-x-100", base)
        }
    };
    let expanding_dot_class = move || {
        let base = "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 \
            flex items-center justify-center rounded-full bg-teal-400 text-white\
            shadow-sm overflow-hidden transition-all duration-300 ease-out\
            pointer-events-none group-focus-visible:ring-2\
            group-focus-visible:ring-teal-500 group-focus-visible:ring-offset-1";
        if always_expanded.get() {
            format!("{} w-7 h-7", base)
        } else {
            format!("{} w-2 h-2 group-hover:w-7 group-hover:h-7", base)
        }
    };
    let plus_class = move || {
        let base = "shrink-0 w-4 h-4 transition-opacity duration-300";
        if always_expanded.get() {
            format!("{} opacity-100", base)
        } else {
            format!("{} opacity-0 group-hover:opacity-100", base)
        }
    };
    view! {
        // The whole strip is the button: the hitbox extends above and below the
        // zero-height line, so the click target is the full width and ~24px tall,
        // which includes the expanded dot sitting at its center.
        <button
            type="button"
            aria-label="Insert operation here"
            class="relative w-full h-0 flex items-center justify-center group z-10 \
                   cursor-pointer focus:outline-none"
        >
            // Invisible hitbox (extends above and below)
            <span class="absolute inset-x-0 -top-3 -bottom-3"></span>

            // Animating horizontal line
            <span class=expanding_line_class></span>

            // Expanding dot
            <span class=expanding_dot_class>
                // Plus icon
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5"
                    class=plus_class
                >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                </svg>
            </span>
        </button>
    }
}
