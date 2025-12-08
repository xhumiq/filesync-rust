use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use web_sys::window;
use crate::i18n::{use_i18n, t, Locale};
use crate::langs::toggle_locale;
use crate::app_state::{ use_folder };

#[component]
pub fn TopFolderNav() -> impl IntoView {
    let (audio_dropdown_open, set_audio_dropdown_open) = signal(false);
    
    let i18n = use_i18n();
    let toggle_language = move |_| {
        toggle_locale(i18n, "");
    };
    let navigate = use_navigate();
    let folder = use_folder();
    if folder.get().is_none() {
        navigate("/files", Default::default());
    }
    let folder = folder.get().unwrap();
    let title = if let Some(t) = folder.title.get("en") {
        t.clone()
    } else {
        folder.name.clone()
    };
    view! {
        {/* ==== TOP BAR ==== */}
        <div class="sticky top-0 z-50 flex items-center justify-center px-4 py-1 text-white bg-teal-700 top-bar">
            <div class="flex items-center">
                <h1 class="mr-6 text-xl font-bold"><A href="/">{title.to_string()}</A></h1>   
            </div>

            {/* Mobile Menu Button */}
            <div class="flex ml-auto space-x-2 md:hidden">
                <button class="text-white border-white btn btn-outline btn-sm">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                    </svg>
                </button>
                <button class="text-white border-white btn btn-outline btn-sm" on:click=toggle_language>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
