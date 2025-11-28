use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;
use crate::api::*;
use crate::icons::*;
use crate::models::channel::{Channel, MediaEntry};
use crate::components::main_top_nav::MainTopNav;
use crate::components::calendar::Calendar;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use gloo::utils::document;
use gloo::timers::callback::Timeout;
use web_sys::{ScrollIntoViewOptions, ScrollLogicalPosition};
use crate::i18n::{use_i18n, t};


 fn menu_view(date_map: Option<HashMap<NaiveDate, usize>>, set_selected_date: WriteSignal<Option<NaiveDate>>) -> AnyView {
    let i18n = use_i18n();
    view! {
        <div class="w-full">
            <div class="border border-gray-200 rounded-b-lg" style="max-width: 400px;margin: 0 auto;">
                <div class="flex flex-col justify-center p-4 space-y-2">
                    <A href="/ui/audio/this_week" attr:class="w-full btn btn-lg btn-accent">{t!(i18n, this_week)}</A>
                    <Calendar available_dates=date_map set_selected_date=set_selected_date />
                    <A href="/ui/audio/all" attr:class="w-full btn btn-lg btn-accent">{t!(i18n, all)}</A>
                </div>
            </div>
        </div>
    }.into_any()
}

fn audio_list_view(mut entries: Vec<MediaEntry>) -> AnyView {
    let i18n = use_i18n();
    // Sort entries by pub_date, then by event
    entries.sort_by(|a, b| {
        a.pub_date.date().cmp(&b.pub_date.date()).then(a.event.cmp(&b.event))
    });

    let first_date = entries[0].pub_date.date();
    let prev_date = first_date - chrono::Duration::days(7);
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
                        let today = Utc::now().date_naive();
                        let mut curr_date = None::<NaiveDate>;
                        let mut _curr_event = None::<String>;
                        entries_clone.iter().enumerate().map(|(index, entry)| {
                            let entry = entry.clone();
                            let size_text = format_size(entry.size);
                            let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };
                            let date_header = if Some(entry.pub_date.date()) != curr_date {
                                curr_date = Some(entry.pub_date.date());
                                let date_str = if crate::get_current_language_code() == "zh" {
                                    entry.pub_date.date().format("%Y年%m月%d日 %A").to_string()
                                        .replace("Monday", "星期一")
                                        .replace("Tuesday", "星期二")
                                        .replace("Wednesday", "星期三")
                                        .replace("Thursday", "星期四")
                                        .replace("Friday", "星期五")
                                        .replace("Saturday", "星期六")
                                        .replace("Sunday", "星期日")
                                } else {
                                    entry.pub_date.date().format("%A, %B %e, %Y").to_string()
                                };
                                Some(view! {
                                    <div id={format!("date-{}", entry.pub_date.date().format("%Y%m%d"))} class="flex items-center justify-between px-4 py-2 text-lg font-bold text-gray-800 bg-gray-200 border-b">
                                        <span>{date_str}</span>
                                        <div class="flex items-center gap-2">
                                            {if entry.pub_date.date() == first_date || entry.pub_date.date() == last_date {
                                                view! {
                                                    <A href=format!("/ui/audio/{}", prev_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                                        {t!(i18n, past_week)}
                                                    </A>
                                                }.into_any()
                                            } else{
                                                view! { <></> }.into_any()
                                            }}
                                            {if next_date <= today && (entry.pub_date.date() == first_date || entry.pub_date.date() == last_date) {
                                                view! {
                                                    <A href=format!("/ui/audio/{}", next_date.format("%y%m%d")) attr:class="btn btn-sm btn-ghost">
                                                        {t!(i18n, next_week)}
                                                    </A>
                                                }.into_any()
                                            } else{
                                                view! { <></> }.into_any()
                                            }}
                                            <A href="/ui/audio/date" attr:class="btn btn-sm btn-ghost" attr:style="padding-x:15px;">
                                                {calendar_icon()}
                                            </A>
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            };

                            let fname = entry.file_name.clone();
                            let fname_for_href = fname.clone();
                            let media_link = entry.link.clone();
                            view! {
                                <>
                                    {date_header}
                                    <a href=format!("{}", media_link) onclick="event.stopPropagation(); return true;" class=format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)>
                                        <div class="flex items-center flex-1 min-w-0">
                                            <span style="margin-left: 15px;margin-right: 0.6rem;">{audio_icon()}</span>
                                            <span class="truncate" style="min-width: 10rem;margin-right: 0.5rem;">{fname}</span>
                                            <span class="truncate">{entry.description}</span>
                                        </div>
                                        <div class="text-sm text-right text-gray-600" style="min-width: 8rem;">
                                            <span class="truncate" style="display: inline-block;min-width: 4rem;padding-right: 0.5rem;">{size_text}</span>
                                            <A href=format!("{}/Audio/chinese/{}", get_api_file_listing_url(), fname_for_href.as_str()) attr:class="btn btn-sm btn-ghost" attr:style="padding:0;width:2rem;">
                                                {chinese_icon()}
                                            </A>
                                            <A href=format!("{}/Audio/english/{}", get_api_file_listing_url(), fname_for_href.as_str()) attr:class="btn btn-sm btn-ghost" attr:style="padding:0;width:2rem;">
                                                {english_icon()}
                                            </A>
                                        </div>
                                    </a>
                                </>
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
pub fn AudioView() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let navigate_for_fetch = navigate.clone();
    let navigate_for_effect = navigate.clone();
    let _navigate_for_view = navigate.clone();
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
    let (_date_range, set_date_range) = signal(Option::<(NaiveDate, NaiveDate)>::None);

    /* ----------------------------------------------------------- */
    /*  Effect: fetch the channel                                  */
    /* ----------------------------------------------------------- */
    Effect::new(move |_| {
        set_loading.set(true);
        set_error.set(String::new());
        let nav = navigate_for_fetch.clone();

        spawn_local(async move {
            let lang_code = crate::get_current_language_code();
            match fetch_files(format!("{}/audio-chi", lang_code)).await {
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
    let url_prefix = "/ui/audio".to_string();
    Effect::new(move |_| {
        if let Some(date) = selected_date.get() {
            navigate_for_effect(&format!("{}/{}", url_prefix, date.format("%y%m%d")), Default::default());
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
                set_date_range.set(Option::None);
            } else {
                let ents = if p == "this_week" {
                    let today = Utc::now().date_naive();
                    let start = today - chrono::Duration::days(6);
                    let end = today;
                    set_selected_date.set(Some(today));
                    ch.date_range(start, end)
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
                } else if p == "all" {
                    ch.entries.clone()
                } else if p.len() == 6 && p.chars().all(|c| c.is_digit(10)) {
                    if let Ok(mut date) = NaiveDate::parse_from_str(&p, "%y%m%d") {
                        let mut start = date;
                        let mut end = date + chrono::Duration::days(6);
                        let today = Utc::now().date_naive();
                        if end > today {
                            end = today;
                            start = today - chrono::Duration::days(6);
                            set_selected_date.set(Some(today));
                            date = today;
                        }else{
                            set_selected_date.set(Some(start));
                        }
                        let ents = ch.date_range(start, end);
                        set_entries.set(ents);
                        Timeout::new(100, move || {
                            if let Some(el) = document().get_element_by_id(&format!("date-{}", date.format("%Y%m%d"))) {
                                let options = ScrollIntoViewOptions::new();
                                options.set_block(ScrollLogicalPosition::Center);
                                let _ = el.scroll_into_view_with_scroll_into_view_options(&options);
                            }
                        }).forget();
                        return;
                    } else {
                        Vec::new()
                    }
                } else if p.contains('/') {
                    let parts: Vec<&str> = p.split('/').collect();
                    if parts.len() == 2 {
                        if let Ok(mut date) = NaiveDate::parse_from_str(parts[0], "%y%m%d") {
                            if let Ok(days) = parts[1].parse::<u32>() {
                                let mut start = date;
                                let mut end = start + chrono::Duration::days(days as i64 - 1);
                                let today = Utc::now().date_naive();
                                if end > today {
                                    end = today;
                                    start = (end - chrono::Duration::days(days as i64 - 1)).max(date);
                                    set_selected_date.set(Some(today));
                                    date = today;
                                }else{
                                    set_selected_date.set(Some(start));
                                }
                                let ents = ch.date_range(start, end);
                                set_entries.set(ents);
                                Timeout::new(100, move || {
                                    if let Some(el) = document().get_element_by_id(&format!("date-{}", date.format("%Y%m%d"))) {
                                        let options = ScrollIntoViewOptions::new();
                                        options.set_block(ScrollLogicalPosition::Center);
                                        let _ = el.scroll_into_view_with_scroll_into_view_options(&options);
                                    }
                                }).forget();
                                return;
                            } else if let Ok(end_date) = NaiveDate::parse_from_str(parts[1], "%y%m%d") {
                                let mut start = date.min(end_date);
                                let mut end = date.max(end_date);
                                let today = Utc::now().date_naive();
                                if end > today {
                                    end = today;
                                    start = today - chrono::Duration::days(6);
                                    set_selected_date.set(Some(today));
                                    date = today;
                                }else{
                                    set_selected_date.set(Some(start));
                                }
                                let ents = ch.date_range(start, end);
                                set_entries.set(ents);
                                Timeout::new(100, move || {
                                    if let Some(el) = document().get_element_by_id(&format!("date-{}", date.format("%Y%m%d"))) {
                                        let options = ScrollIntoViewOptions::new();
                                        options.set_block(ScrollLogicalPosition::Center);
                                        let _ = el.scroll_into_view_with_scroll_into_view_options(&options);
                                    }
                                }).forget();
                                return;
                            }
                        }
                        Vec::new()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                if ents.len() > 0 {
                    set_date_range.set(Some((ents[0].pub_date.date(), ents[ents.len()-1].pub_date.date())));
                }else{
                    set_date_range.set(Option::None);
                }
                set_entries.set(ents);
            }
        } else {
            set_entries.set(Vec::new());
            set_date_range.set(Option::None);
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
                    let entries = entries.get();
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
                                {t!(i18n, ntc_audio)}
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
                                {t!(i18n, ntc_audio)}
                            </h3>
                            {menu_view(date_map.get(), set_selected_date)}
                        }.into_any()
                    } else {
                        if entries.is_empty() {
                            if path()!="" {
                                view! {
                                    <>
                                        <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                            {t!(i18n, ntc_audio)}
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
                            view!{
                                <>
                                    <h3 class="pb-2 text-4xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';margin-bottom: 1rem;">
                                        {t!(i18n, ntc_audio)}
                                    </h3>
                                    {audio_list_view(entries)}
                                </>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </>
    }
}