use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::*;
use crate::icons::*;
use crate::models::channel::{Channel, MediaEntry};
use chrono::{NaiveDate, Weekday, Datelike};
use std::sync::Arc;

fn file_list_view(entries: Vec<MediaEntry>) -> AnyView {
    view! {
        <div class="w-full">
            // Scrollable container
            <div class="border border-gray-200 rounded-b-lg">
                {move || {
                    if entries.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-32 text-gray-500">
                                "No files found"
                            </div>
                        }.into_any()
                    } else {
                        entries.iter().enumerate().map(|(index, entry)| {
                            let entry = entry.clone();
                            let size_text = format_size(entry.size);
                            let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };

                            view! {
                                <div class={format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)}>
                                    <div class="flex items-center flex-1 min-w-0">
                                        {film_icon()}
                                        <span class="truncate">{entry.file_name}</span>
                                    </div>
                                    <div class="w-24 text-sm text-right text-gray-600">
                                        {size_text}
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }.into_any()
}

/* --------------------------------------------------------------- */
/*  Main component                                                */
/* --------------------------------------------------------------- */
#[component]
pub fn Folder() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let path = move || {
        params
            .with(|p| p.get("path").map(|s| s.clone()))
            .unwrap_or_default()
    };

    let (channel, set_channel) = signal(Option::<Channel>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());
    let (weeks, set_weeks) = signal(Option::<Vec<(NaiveDate, NaiveDate)>>::None);

    /* ----------------------------------------------------------- */
    /*  Effect: fetch whenever the route path changes               */
    /* ----------------------------------------------------------- */
    Effect::new(move |_| {
        let cur = path();
        if cur.is_empty() {
            return;
        }

        set_loading.set(true);
        set_error.set(String::new());

        spawn_local(async move {
            match fetch_files(cur).await {
                Ok(ch) => set_channel.set(Some(ch)),
                Err(e) => set_error.set(e.to_string()),
            }
            set_loading.set(false);
        });
    });

    /* ----------------------------------------------------------- */
    /*  Render                                                     */
    /* ----------------------------------------------------------- */
    view! {
        <>
            {/* ==== TOP BAR ==== */}
            <div class="flex items-center justify-between px-0 py-0 text-white bg-teal-700 top-bar">
                <h1 class="text-xl font-bold">ZSF {path}</h1>
                <div class="flex space-x-2">
                    <button class="text-white border-white btn btn-outline btn-sm">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                        </svg>
                    </button>
                    <button class="text-white border-white btn btn-outline btn-sm">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9" />
                        </svg>
                    </button>
                </div>
            </div>

            {/* ==== MAIN CONTENT ==== */}
            <div class="container p-4 mx-auto">

                {move || {
                    if loading.get() {
                        // DaisyUI spinner
                        view! {
                            <div class="flex justify-center py-8">
                                <span class="loading loading-spinner loading-lg"></span>
                            </div>
                        }.into_any()
                    } else if !error.get().is_empty() {
                        // DaisyUI alert
                        view! {
                            <div class="shadow-lg alert alert-error">
                                <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-6 h-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                                <span>{error.get()}</span>
                            </div>
                        }.into_any()
                    } else {
                        if let Some(ch) = channel.get() {
                            file_list_view(ch.entries)
                        } else {
                            view! {
                                <div class="flex items-center justify-center h-32 text-gray-500">
                                    "No data available"
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </>
    }
}