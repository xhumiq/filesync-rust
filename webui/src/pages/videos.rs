use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;
use crate::api::*;
use crate::icons::*;
use crate::models::channel::{Channel, MediaEntry};
use crate::components::main_top_nav::MainTopNav;
use crate::components::calendar::Calendar;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use crate::i18n::{use_i18n, t};

fn menu_view(date_map: Option<HashMap<NaiveDate, usize>>, set_selected_date: WriteSignal<Option<NaiveDate>>) -> AnyView {
    let i18n = use_i18n();
    view! {
        <div class="w-full">
            <div class="border border-gray-200 rounded-b-lg" style="max-width: 400px;margin: 0 auto;">
                <div class="flex flex-col justify-center p-4 space-y-2">
                    <A href="/ui/audio/today" attr:class="w-full btn btn-lg btn-accent">{t!(i18n, today)}</A>
                    <Calendar available_dates=date_map set_selected_date=set_selected_date />
                </div>
            </div>
        </div>
    }.into_any()
}

fn video_list_view(mut entries: Vec<MediaEntry>) -> AnyView {
    let i18n = use_i18n();
    // Sort entries by pub_date, then by event
    entries.sort_by(|a, b| {
        a.pub_date.date().cmp(&b.pub_date.date()).then(a.event.cmp(&b.event)).then(a.index.cmp(&b.index))
    });
    let first_date = entries[0].pub_date.date();
    let prev_date = first_date - chrono::Duration::days(1);
    let last_date = entries[entries.len()-1].pub_date.date();
    let next_date = last_date + chrono::Duration::days(1);
    view! {
        <div id="segmented-list" class="w-full">
            <div class="border border-gray-200 rounded-b-lg">
                {
                let entries_clone = entries.clone();
                move || {
                    if entries_clone.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-32 text-gray-500">
                                {t!(i18n, no_files_found)}
                            </div>
                        }.into_any()
                    } else {
                        let mut curr_date = None::<NaiveDate>;
                        let mut curr_event = None::<String>;
                        let today = Utc::now().date_naive();
                        entries_clone.iter().enumerate().map(|(index, entry)| {
                            let entry = entry.clone();
                            let size_text = format_size(entry.size);
                            let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };

                            let date_header = if Some(entry.pub_date.date()) != curr_date {
                                curr_date = Some(entry.pub_date.date());
                                Some(view! {
                                    <div class="flex items-center justify-between py-2 text-lg font-bold text-gray-800 bg-gray-200 border-b" style="padding-left: 15px;">
                                        <span>{entry.pub_date.date().format("%A, %B %e, %Y").to_string()}</span>
                                        <div class="flex items-center gap-2">
                                            {if entry.pub_date.date() == first_date {
                                                view! {
                                                    <A href=format!("/ui/videos/{}", prev_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                                        {t!(i18n, previous_day)}
                                                    </A>
                                                }.into_any()
                                            } else{
                                                view! { <></> }.into_any()
                                            }}
                                            {if next_date <= today && entry.pub_date.date() == last_date {
                                                view! {
                                                    <A href=format!("/ui/videos/{}", next_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                                        {t!(i18n, next_day)}
                                                    </A>
                                                }.into_any()
                                            } else{
                                                view! { <></> }.into_any()
                                            }}
                                            <A href="/ui/videos/date" attr:class="btn btn-sm btn-ghost" attr:style="padding-x:15px;">
                                                {calendar_icon()}
                                            </A>
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            };

                            let event_header = if Some(entry.event.clone()) != curr_event {
                                curr_event = Some(entry.event.clone());
                                Some(view! {
                                    <h4 class="px-4 py-1 font-semibold text-gray-700 bg-gray-100 border-b text-md">
                                        <span class="mr-2">{entry.pub_date.date().format("%m.%d").to_string()}</span><span class="mr-2">{entry.event}</span><span class="mr-2">{entry.event_desc}</span>
                                    </h4>
                                })
                            } else {
                                None
                            };

                            view! {
                                <>
                                    {date_header}
                                    {event_header}
                                    <div class={format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)}>
                                        <div class="flex items-center flex-1 min-w-0">
                                            <span style="margin-left: 15px;margin-right: 10px;">{film_icon()}</span>
                                            <span class="truncate">{entry.file_name}</span>
                                        </div>
                                        <div class="w-24 text-sm text-right text-gray-600">
                                            {size_text}
                                        </div>
                                    </div>
                                </>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
            <div class="border-t border-gray-200 rounded-b-lg">
                {move || {
                    if !entries.is_empty() {
                        let entry = &entries[0];
                        let today = Utc::now().date_naive();
                        Some(view! {
                            <div class="flex items-center justify-between py-2 text-lg font-bold text-gray-800 bg-gray-200 border-b" style="padding-left: 15px;">
                                <span>{entry.pub_date.date().format("%A, %B %e, %Y").to_string()}</span>
                                <div class="flex items-center gap-2">
                                    <A href=format!("/ui/videos/{}", prev_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                        {t!(i18n, previous_day)}
                                    </A>
                                    {if next_date <= today {
                                        view! {
                                            <A href=format!("/ui/videos/{}", next_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                                {t!(i18n, next_day)}
                                            </A>
                                        }.into_any()
                                    } else{
                                        view! { <></> }.into_any()
                                    }}
                                    <A href="/ui/videos/date" attr:class="btn btn-sm btn-ghost" attr:style="padding-x:15px;">
                                        {calendar_icon()}
                                    </A>
                                </div>
                            </div>
                        })
                    } else {
                        None
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
pub fn VideoView() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let navigate_for_fetch = navigate.clone();
    let params = leptos_router::hooks::use_params_map();
    let path = move || {
        params
            .with(|p| p.get("path").map(|s| s.clone()))
            .unwrap_or_default()
    };

    let (channel, set_channel) = signal(Option::<Channel>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(String::new());
    let (entries, set_entries) = signal(Vec::<MediaEntry>::new());
    let (date_map, set_date_map) = signal(Option::<HashMap<NaiveDate, usize>>::None);
    let (selected_date, set_selected_date) = signal(None::<NaiveDate>);

    /* ----------------------------------------------------------- */
    /*  Effect: fetch the channel                                   */
    /* ----------------------------------------------------------- */
    Effect::new(move |_| {
        let nav = navigate_for_fetch.clone();
        set_loading.set(true);
        set_error.set(String::new());

        spawn_local(async move {
            match fetch_files("zh/videos-all".to_string()).await {
                Ok(ch) => {
                    let mut map = HashMap::new();
                    for entry in &ch.entries {
                        *map.entry(entry.pub_date.date()).or_insert(0) += 1;
                    }
                    set_channel.set(Some(ch));
                    set_date_map.set(Some(map));
                },
                Err(e) => {
                    if e.to_string().contains("JWT token") {
                        nav("/account/login", Default::default());
                        //return;
                    }
                    set_error.set(e.to_string());
                }
            }
            set_loading.set(false);
        });
    });

    /* ----------------------------------------------------------- */
    /*  Effect: navigate on date selection                         */
    /* ----------------------------------------------------------- */
    let url_prefix = "/ui/videos".to_string();
    Effect::new(move |_| {
        if let Some(date) = selected_date.get() {
            navigate(&format!("{}/{}", url_prefix, date.format("%y%m%d")), Default::default());
            set_selected_date.set(None);
        }
    });

    /* ----------------------------------------------------------- */
    /*  Effect: set entries based on path and channel             */
    /* ----------------------------------------------------------- */
    Effect::new(move |_| {
        let p = path();
        if let Some(ch) = channel.get() {
            if p == "date" {
                // Create date map
                set_entries.set(Vec::new());
            } else {
                let ents = if p == "today" {
                    ch.entries_for_today()
                } else if p.ends_with("days") {
                    if let Ok(x_str) = p.trim_end_matches("days").parse::<u32>() {
                        if x_str >= 1 && x_str <= 9 {
                            let today = Utc::now().date_naive();
                            let start = today - chrono::Duration::days(x_str as i64 - 1);
                            let end = today;
                            ch.date_range(start, end)
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else if p.len() == 6 && p.chars().all(|c| c.is_digit(10)) {
                    if let Ok(date) = NaiveDate::parse_from_str(&p, "%y%m%d") {
                        ch.entries_for_date(date)
                    } else {
                        Vec::new()
                    }
                } else if p.contains('/') {
                    let parts: Vec<&str> = p.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(d1), Ok(d2)) = (NaiveDate::parse_from_str(parts[0], "%y%m%d"), NaiveDate::parse_from_str(parts[1], "%y%m%d")) {
                            let start = d1.min(d2);
                            let end = d1.max(d2);
                            ch.date_range(start, end)
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    // default: all entries
                    Vec::new()
                };
                set_entries.set(ents);
            }
        } else {
            set_entries.set(Vec::new());
        }
    });

    /* ----------------------------------------------------------- */
    /*  Render                                                     */
    /* ----------------------------------------------------------- */
    view! {
        <>
            <MainTopNav />

            {/* ==== MAIN CONTENT ==== */}
            <div class="container p-4 mx-auto">
                {move || {
                    let entries=entries.get();
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
                            <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                {t!(i18n, ntc_video)}
                            </h3>
                            <div class="shadow-lg alert alert-error">
                                <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-6 h-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                                <span>{error.get()}</span>
                            </div>
                            {menu_view(date_map.get(), set_selected_date)}
                        }.into_any()
                    } else if path() == "date" {
                        view! {
                            <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                {t!(i18n, ntc_video)}
                            </h3>
                            {menu_view(date_map.get(), set_selected_date)}
                        }.into_any()
                    } else {
                        if entries.is_empty() {
                            if path()!="" {
                                view! {
                                    <>
                                        <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                            {t!(i18n, ntc_video)}
                                        </h3>
                                        <div class="flex justify-center">
                                            <div class="alert alert-info">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-6 h-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                                </svg>
                                                <span>{t!(i18n, no_entries_in_date_range)}</span>
                                            </div>
                                        </div>
                                        {menu_view(date_map.get(), set_selected_date)}
                                    </>
                                }.into_any()
                            } else {
                                menu_view(date_map.get(), set_selected_date)
                            }
                        }else{
                            let next_date = entries[entries.len()-1].pub_date.date() + chrono::Duration::days(1);
                            let today = Utc::now().date_naive();
                            view!{
                                <>
                                    <div class="flex justify-center mb-4">
                                        <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                            {t!(i18n, ntc_video)}
                                        </h3>
                                    </div>
                                    {video_list_view(entries)}
                                </>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </>
    }
}
