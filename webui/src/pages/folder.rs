use leptos::prelude::*;
use serde::Deserialize;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::models::channel::Channel;
use chrono::{NaiveDate, Weekday, Datelike};

fn list_weeks_in_range(start_date: NaiveDate, end_date: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let mut weeks = Vec::new();
    let mut current = start_date;

    while current <= end_date {
        // Find the Saturday of the current week (or end_date if earlier)
        let days_to_saturday = (6 - current.weekday().num_days_from_sunday()) as i64;
        let week_end = current + chrono::Duration::days(days_to_saturday);
        let actual_end = if week_end > end_date { end_date } else { week_end };

        weeks.push((current, actual_end));

        // If we've reached the end date, we're done
        if actual_end >= end_date {
            break;
        }

        // Move to the next Sunday
        current = actual_end + chrono::Duration::days(1);
        // If current is not Sunday, find the next Sunday
        if current.weekday() != Weekday::Sun {
            let days_to_sunday = (7 - current.weekday().num_days_from_sunday()) % 7;
            current = current + chrono::Duration::days(days_to_sunday as i64);
        }
    }

    weeks
}

fn get_api_file_listing_url() -> String {
    std::env::var("API_FILE_LISTING_URL")
        .unwrap_or_else(|_| "http://localhost:3000/files".to_string())
}

fn get_jwt_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("jwt_token").ok().flatten())
}

#[derive(Deserialize, Clone, Debug)]
struct FileItem {
    name: String,
    is_folder: bool,
    size: u64,
}

/* --------------------------------------------------------------- */
/*  Helper: format bytes → KB/MB/GB (1 decimal)                    */
/* --------------------------------------------------------------- */
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/* --------------------------------------------------------------- */
/*  Async fetch (unchanged, only minor error handling)            */
/* --------------------------------------------------------------- */
async fn fetch_files(path: String) -> AnyhowResult<Channel> {
    let url = format!(
        "{}/{}",
        get_api_file_listing_url(),
        path.trim_start_matches('/')
    );

    let jwt = get_jwt_token().ok_or_else(|| anyhow!("No JWT token found"))?;

    web_sys::console::log_1(&format!("Fetch Files from Url: {}", url).into());

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {jwt}"))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e:?}"))?;

    if !resp.ok() {
        if resp.status() == 401 {
            // Redirect to login page on 401 Unauthorized
            if let Some(window) = web_sys::window() {
                if let Some(location) = window.location().href().ok() {
                    let _ = window.location().set_href("/account/login");
                }
            }
            return Err(anyhow!("Unauthorized - redirecting to login"));
        }
        return Err(anyhow!("HTTP {} {}", resp.status(), resp.status_text()));
    }

    resp.json::<Channel>()
        .await
        .map_err(|e| anyhow!("JSON error: {e:?}"))
}

/* --------------------------------------------------------------- */
/*  Folder icon SVG (DaisyUI‑style)                               */
/* --------------------------------------------------------------- */
fn folder_icon_org() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="inline-block w-5 h-5 mr-2 text-amber-600"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M3 7h2l1.5-3h7l1.5 3h5a1 1 0 011 1v10a1 1 0 01-1 1H4a1 1 0 01-1-1V8a1 1 0 011-1z"
            />
        </svg>
    }
}

fn folder_icon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="inline-block w-5 h-5 mr-2 text-amber-600">
            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
        </svg>
    }
}

fn film_icon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="inline-block w-5 h-5 mr-2 text-amber-600">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.375 19.5h17.25m-17.25 0a1.125 1.125 0 0 1-1.125-1.125M3.375 19.5h1.5C5.496 19.5 6 18.996 6 18.375m-3.75 0V5.625m0 12.75v-1.5c0-.621.504-1.125 1.125-1.125m18.375 2.625V5.625m0 12.75c0 .621-.504 1.125-1.125 1.125m1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125m0 3.75h-1.5A1.125 1.125 0 0 1 18 18.375M20.625 4.5H3.375m17.25 0c.621 0 1.125.504 1.125 1.125M20.625 4.5h-1.5C18.504 4.5 18 5.004 18 5.625m3.75 0v1.5c0 .621-.504 1.125-1.125 1.125M3.375 4.5c-.621 0-1.125.504-1.125 1.125M3.375 4.5h1.5C5.496 4.5 6 5.004 6 5.625m-3.75 0v1.5c0 .621.504 1.125 1.125 1.125m0 0h1.5m-1.5 0c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125m1.5-3.75C5.496 8.25 6 7.746 6 7.125v-1.5M4.875 8.25C5.496 8.25 6 8.754 6 9.375v1.5m0-5.25v5.25m0-5.25C6 5.004 6.504 4.5 7.125 4.5h9.75c.621 0 1.125.504 1.125 1.125m1.125 2.625h1.5m-1.5 0A1.125 1.125 0 0 1 18 7.125v-1.5m1.125 2.625c-.621 0-1.125.504-1.125 1.125v1.5m2.625-2.625c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125M18 5.625v5.25M7.125 12h9.75m-9.75 0A1.125 1.125 0 0 1 6 10.875M7.125 12C6.504 12 6 12.504 6 13.125m0-2.25C6 11.496 5.496 12 4.875 12M18 10.875c0 .621-.504 1.125-1.125 1.125M18 10.875c0 .621.504 1.125 1.125 1.125m-2.25 0c.621 0 1.125.504 1.125 1.125m-12 5.25v-5.25m0 5.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125m-12 0v-1.5c0-.621-.504-1.125-1.125-1.125M18 18.375v-5.25m0 5.25v-1.5c0-.621.504-1.125 1.125-1.125M18 13.125v1.5c0 .621.504 1.125 1.125 1.125M18 13.125c0-.621.504-1.125 1.125-1.125M6 13.125v1.5c0 .621-.504 1.125-1.125 1.125M6 13.125C6 12.504 5.496 12 4.875 12m-1.5 0h1.5m-1.5 0c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125M19.125 12h1.5m0 0c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125m-17.25 0h1.5m14.25 0h1.5" />
        </svg>
    }
}

/* --------------------------------------------------------------- */
/*  Main component                                                */
/* --------------------------------------------------------------- */
#[component]
pub fn Folder() -> impl IntoView {
    web_sys::console::log_1(&"Folder function called".into());
    let params = leptos_router::hooks::use_params_map();
    let path = move || {
        params
            .with(|p| p.get("path").map(|s| s.clone()))
            .unwrap_or_default()
    };

    let (channel, set_channel) = signal(Option::<Channel>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());

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
            web_sys::console::log_1(&"Calling Fetch Files in spawn local".into());
            match fetch_files(cur).await {
                Ok(ch) => set_channel.set(Some(ch)),
                Err(e) => set_error.set(e.to_string()),
            }
            set_loading.set(false);
        });
    });
    println!("render path: {}", path());

    // Check date range and print week ranges if conditions are met
    if let Some(ch) = channel.get() {
        if !ch.entries.is_empty() {
            // Entries are sorted in desc order, so first is most recent, last is oldest
            if let (Some(first_entry), Some(last_entry)) = (ch.entries.first(), ch.entries.last()) {
                let date_range_days = (first_entry.pub_date - last_entry.pub_date).num_days();
                if date_range_days > 7 && first_entry.file_name.starts_with("zsv") {
                    println!("Date range over 7 days ({} days) and first filename starts with 'zsv'", date_range_days);
                    println!("Entry date range: {} to {}", last_entry.pub_date, first_entry.pub_date);

                    let weeks = list_weeks_in_range(last_entry.pub_date, first_entry.pub_date);
                    println!("Week ranges:");
                    for (week_num, (week_start, week_end)) in weeks.iter().enumerate() {
                        println!("  Week {}: {} ({}) to {} ({})",
                            week_num + 1,
                            week_start,
                            week_start.format("%A"),
                            week_end,
                            week_end.format("%A")
                        );
                    }
                }
            }
        }
    }

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
                        // Virtual Scrollable List
                        view! {
                            <div class="w-full">
                                // Header row
                                <div class="flex items-center px-4 py-2 text-sm font-semibold text-gray-700 bg-gray-100 border-b">
                                    <div class="flex-1">"Name"</div>
                                    <div class="w-24 text-right">"Size"</div>
                                </div>

                                // Scrollable container
                                <div class="overflow-y-auto border border-gray-200 rounded-b-lg" style="height: 400px;">
                                    {move || {
                                        match channel.get() {
                                            Some(ch) => {
                                                if ch.entries.is_empty() {
                                                    view! {
                                                        <div class="flex items-center justify-center h-32 text-gray-500">
                                                            "No files found"
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    ch.entries.into_iter().enumerate().map(|(index, entry)| {
                                                        let size_text = format_size(entry.size);
                                                        let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };

                                                        view! {
                                                            <div class={format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)}>
                                                                <div class="flex items-center flex-1 min-w-0">
                                                                    {film_icon()}
                                                                    <span class="truncate">{entry.file_name.clone()}</span>
                                                                </div>
                                                                <div class="w-24 text-sm text-right text-gray-600">
                                                                    {size_text}
                                                                </div>
                                                            </div>
                                                        }
                                                    }).collect_view().into_any()
                                                }
                                            }
                                            None => {
                                                view! {
                                                    <div class="flex items-center justify-center h-32 text-gray-500">
                                                        "No data available"
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </>
    }
}