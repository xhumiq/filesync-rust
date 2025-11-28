use std::cmp::Ordering;

use leptos::prelude::*;
use leptos_router::components::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::*;
use crate::icons::*;
use crate::models::channel::{Channel, MediaEntry};
use chrono::NaiveDate;
use crate::components::main_top_nav::MainTopNav;
use crate::i18n::{use_i18n, t};

fn breadcrumb_view(path: &str) -> impl IntoView {
    let i18n = use_i18n();
    
    // Split path into segments and create breadcrumb links
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    
    view! {
        <div class="flex items-center justify-between w-full py-2 text-lg font-bold text-gray-800 bg-gray-200 border-b" style="padding-left: 15px;">
            <ol class="flex items-center space-x-2">
                // Home/Root link
                <li class="flex items-center">
                    <A href="/" attr:class="flex items-center text-sm font-bold text-gray-700 hover:text-blue-600">
                        {t!(i18n, folder)}
                    </A>
                </li>
                
                // Path segments
                {segments.iter().enumerate().map(|(index, segment)| {
                    let is_last = index == segments.len() - 1;
                    let path_up_to_here = segments[0..=index].join("/");
                    let href = format!("/files/{}", path_up_to_here);
                    let segment_text = segment.to_string();
                    
                    view! {
                        <li class="flex items-center">
                            <svg class="w-3 h-3 mx-1 text-gray-400" style="max-width: 12px; max-height: 12px; height:20px; width:20px; margin-right:0.6rem;" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 6 10">
                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 9 4-4-4-4"/>
                            </svg>
                            {if is_last {
                                view! {
                                    <span class="text-sm font-bold" style="color: #900;">{segment_text.clone()}</span>
                                }.into_any()
                            } else {
                                view! {
                                    <A href=href attr:class="text-sm font-bold text-gray-700 hover:text-blue-600">{segment_text.clone()}</A>
                                }.into_any()
                            }}
                        </li>
                    }
                }).collect_view()}
            </ol>
        </div>
    }
}

fn file_list_view(path: &str, entries: Vec<MediaEntry>) -> AnyView {
    let i18n = use_i18n();
    let mut entries = entries.clone();
    entries.sort_by(|a, b| {
        if (a.content_type == "folder" || b.content_type == "folder") && a.content_type != b.content_type {
            return if a.content_type == "folder" { Ordering::Less } else { Ordering::Greater };
        }
        if a.content_type == "folder" {
            return a.title.to_lowercase().cmp(&b.title.to_lowercase());
        }else{
            return a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase());
        }
    });
    let path = path.to_string();
    view! {
        <div class="w-full">
            // Scrollable container
            <div class="border border-gray-200 rounded-b-lg">
                {move || {
                    if entries.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-32 text-gray-500">
                                {t!(i18n, no_files_found)}
                            </div>
                        }.into_any()
                    } else {
                        entries.iter().enumerate().map(|(index, entry)| {
                            let entry = entry.clone();
                            let size_text = format_size(entry.size);
                            let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };
                            if entry.content_type == "folder" {
                                view! {
                                    <A href=format!("/files/{}/{}", path, entry.title) attr:class=format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)>
                                        <div class="flex items-center flex-1 min-w-0">
                                            <span style="margin-left: 15px;margin-right: 10px;max-width: 20px;max-height: 20px"><MimeTypeIcon content_type=entry.content_type.clone() mime_type=entry.mime_type.clone() /></span>
                                            <span class="truncate">{entry.title}</span>
                                        </div>
                                        <div class="w-24 text-sm text-right text-gray-600">
                                        </div>
                                    </A>
                                }.into_any()
                            }else{
                                let fname = entry.file_name.clone();
                                let fname_for_href = fname.clone();
                                view! {
                                    <a href=format!("{}/{}/{}", get_api_file_listing_url(), path, &fname_for_href) onclick="event.stopPropagation(); return true;" class=format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)>
                                        <div class="flex items-center flex-1 min-w-0">
                                            <span style="margin-left: 15px;margin-right: 10px;max-width: 20px;max-height: 20px"><MimeTypeIcon content_type=entry.content_type.clone() mime_type=entry.mime_type.clone() /></span>
                                            <span class="truncate">{fname}</span>
                                        </div>
                                        <div class="w-24 text-sm text-right text-gray-600">
                                            {size_text}
                                        </div>
                                    </a>
                                }.into_any()
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
    let (_weeks, _set_weeks) = signal(Option::<Vec<(NaiveDate, NaiveDate)>>::None);

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
            <MainTopNav />

            {/* ==== MAIN CONTENT ==== */}
            <div class="container p-4 mx-auto">

                {/* ==== BREADCRUMBS ==== */}
                {move || {
                    let current_path = path();
                    if !current_path.is_empty() {
                        breadcrumb_view(&current_path).into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            
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
                            let current_path = path();
                            file_list_view(&current_path, ch.entries)
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