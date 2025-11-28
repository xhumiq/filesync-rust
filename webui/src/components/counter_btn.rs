use leptos::prelude::*;

/// A parameterized incrementing button
#[component]
pub fn Button(#[prop(default = 1)] increment: i32) -> impl IntoView {
    let count = RwSignal::new(0);
    view! {
        <button on:click=move |_| {
            count.update(|c| *c += increment);
        }>
            "Click me: " {count} " (+" {increment} ")"
        </button>
    }
}
