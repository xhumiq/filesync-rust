use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use web_sys::window;
use crate::i18n::{use_i18n, t, Locale};
use crate::langs::toggle_locale;
use crate::app_state::{ use_folder, use_app_state, logout };

#[component]
pub fn TopFolderNav() -> impl IntoView {
    let (audio_dropdown_open, set_audio_dropdown_open) = signal(false);

    let i18n = use_i18n();
    let current_locale = Memo::new(move |_| i18n.get_locale());
    let toggle_language = move |_| {
        toggle_locale(i18n, "");
    };
    let navigate = use_navigate();
    let app_state = use_app_state();
    let folder = use_folder();
    let (title, set_title) = signal(String::new());
    
    let on_logout = move |_| {
        logout(&app_state);
    };

    Effect::new(move |_| {
        if folder.get().is_none() {
            navigate("/files", Default::default());
        }
        let locale = current_locale.get().to_string();
        set_title.set(folder.get().map(|f| f.title.get(&locale).unwrap_or(&f.name).clone()).unwrap_or_default());
    });
    view! {
        <div class="relative sticky top-0 z-50 flex items-center justify-center px-4 py-1 text-white bg-teal-700 top-bar">
            <h1 class="text-xl font-bold"><A href="/browse">{move || title.get()}</A></h1>

            <div class="absolute flex space-x-2 right-4">
                <button class="text-white border-white btn btn-outline btn-sm" on:click=toggle_language>
                    {move || {
                        match current_locale.get() {
                            Locale::en => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="12" font-weight="bold" fill="currentColor">En</text>
                                </svg>
                            }.into_any(),
                            Locale::fr => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="12" font-weight="bold" fill="currentColor">Fr</text>
                                </svg>
                            }.into_any(),
                            Locale::zh => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="14" font-weight="bold" fill="currentColor">文</text>
                                </svg>
                            }.into_any(),
                        }
                    }}
                </button>
                <button class="text-white border-white btn btn-outline btn-sm" on:click=on_logout>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
