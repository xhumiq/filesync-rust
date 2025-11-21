use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;
use crate::api::*;
use crate::icons::*;
use crate::models::channel::{Channel, MediaEntry};
use crate::components::main_top_nav::MainTopNav;
use crate::components::calendar::Calendar;
use chrono::{NaiveDate, Utc, Duration, Datelike};
use std::collections::HashMap;
use std::sync::Arc;
use gloo::utils::document;
use gloo::timers::callback::Timeout;
use web_sys::{ScrollIntoViewOptions, ScrollLogicalPosition};

fn compute_weeks(entries: &[MediaEntry]) -> Vec<(NaiveDate, NaiveDate)> {
    let mut weeks = Vec::new();
    if entries.is_empty() {
        return weeks;
    }
    let mut sorted_dates: Vec<NaiveDate> = entries.iter().map(|e| e.pub_date).collect();
    sorted_dates.sort();
    sorted_dates.dedup();
    let mut current_week_start = None;
    for &date in &sorted_dates {
        let week_start = date - Duration::days(date.weekday().num_days_from_monday() as i64);
        if current_week_start != Some(week_start) {
            if let Some(start) = current_week_start {
                let end = start + chrono::Duration::days(6);
                weeks.push((start, end));
            }
            current_week_start = Some(week_start);
        }
    }
    if let Some(start) = current_week_start {
        let end = start + Duration::days(6);
        weeks.push((start, end));
    }
    weeks
}

 fn menu_view(date_map: Option<HashMap<NaiveDate, usize>>, set_selected_date: WriteSignal<Option<NaiveDate>>) -> AnyView {
    view! {
        <div class="w-full">
            <div class="border border-gray-200 rounded-b-lg" style="max-width: 400px;margin: 0 auto;">
                <div class="flex flex-col justify-center p-4 space-y-2">
                    <A href="/ui/photos/this_week" attr:class="w-full btn btn-lg btn-accent">今天 Today</A>
                    <Calendar available_dates=date_map set_selected_date=set_selected_date />
                </div>
            </div>
        </div>
    }.into_any()
}

fn photo_list_view(mut entries: Vec<MediaEntry>) -> AnyView {
    // Sort entries by pub_date, then by event
    entries.sort_by(|a, b| {
        a.pub_date.cmp(&b.pub_date).then(a.event.cmp(&b.event))
    });

    view! {
        <div id="segmented-list" class="w-full">
            <div class="border border-gray-200 rounded-b-lg">
                {move || {
                    if entries.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-32 text-gray-500">
                                "No files found"
                            </div>
                        }.into_any()
                    } else {
                        let mut prev_date = None::<NaiveDate>;
                        entries.iter().enumerate().map(|(index, entry)| {
                            let entry = entry.clone();
                            let size_text = format_size(entry.size);
                            let bg_class = if index % 2 == 0 { "bg-white" } else { "bg-gray-50" };

                            let date_header = if Some(entry.pub_date) != prev_date {
                                prev_date = Some(entry.pub_date);
                                Some(view! {
                                    <div id={format!("date-{}", entry.pub_date.format("%Y%m%d"))} class="flex items-center justify-between px-4 py-2 text-lg font-bold text-gray-800 bg-gray-200 border-b">
                                        <span>{entry.pub_date.format("%A, %B %e, %Y").to_string()}</span>
                                        <A href="/ui/photos/date" attr:class="btn btn-sm btn-ghost">
                                            {calendar_icon()}
                                        </A>
                                    </div>
                                })
                            } else {
                                None
                            };

                            view! {
                                <>
                                    {date_header}
                                    <A href=format!("http://localhost:3000/fs/v1/Music/ZSF/Chinese/{}", entry.file_name) attr:class=format!("flex items-center px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-100 {}", bg_class)>
                                        <div class="flex items-center flex-1 min-w-0">
                                            {photo_icon()}
                                            <span class="truncate">{
                                                let mut name = entry.location.clone();
                                                name = if name.is_empty() { entry.file_name.clone() } else { name };
                                                let mut index = entry.event_code.clone();
                                                if !index.is_empty() || !entry.event_date_stamp.is_empty() {
                                                    if !index.is_empty() && !entry.event_date_stamp.is_empty() {
                                                        index = format!(" [{}{}]", index, entry.event_date_stamp)
                                                    }else if !index.is_empty(){
                                                        index = format!(" [{}]", index)
                                                    }else if !entry.event_date_stamp.is_empty(){
                                                        index = format!(" [{}]", entry.event_date_stamp)
                                                    }
                                                }
                                                format!("{}{}: {}", name, index, entry.event_desc)
                                            }</span>
                                        </div>
                                        <div class="w-24 text-sm text-right text-gray-600">
                                            {size_text}
                                        </div>
                                    </A>
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
pub fn PhotosView() -> impl IntoView {
    let navigate = Arc::new(use_navigate());
    let navigate_for_effect = navigate.clone();
    let navigate_for_view = navigate.clone();
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
    let (date_range, set_date_range) = signal(Option::<(NaiveDate, NaiveDate)>::None);

    /* ----------------------------------------------------------- */
    /*  Effect: fetch the channel                                   */
    /* ----------------------------------------------------------- */
    Effect::new(move |_| {
        set_loading.set(true);
        set_error.set(String::new());

        spawn_local(async move {
            match fetch_files("Pictures/Chinese".to_string()).await {
                Ok(ch) => {
                    let mut map = HashMap::new();
                    for entry in &ch.entries {
                        *map.entry(entry.pub_date).or_insert(0) += 1;
                    }
                    set_channel.set(Some(ch));
                    set_date_map.set(Some(map));
                },
                Err(e) => set_error.set(e.to_string()),
            }
            set_loading.set(false);
        });
    });

    /* ----------------------------------------------------------- */
    /*  Effect: navigate on date selection                         */
    /* ----------------------------------------------------------- */
    let url_prefix = "/ui/photos".to_string();
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
                            } else if let Ok(mut end_date) = NaiveDate::parse_from_str(parts[1], "%y%m%d") {
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
                    set_date_range.set(Some((ents[0].pub_date, ents[ents.len()-1].pub_date)));
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
                            <div class="shadow-lg alert alert-error">
                                <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-6 h-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                </svg>
                                <span>{error.get()}</span>
                            </div>
                            {menu_view(date_map.get(), set_selected_date)}
                        }.into_any()
                    } else if path() == "date" {
                        menu_view(date_map.get(), set_selected_date)
                    } else {
                        if entries.is_empty() {
                            if path()!="" {
                                view! {
                                    <>
                                        <div class="flex justify-center">
                                            <div class="alert alert-info">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="flex-shrink-0 w-6 h-6 stroke-current" fill="none" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                                </svg>
                                                <span>No photo entries for the selected date range.</span>
                                            </div>
                                        </div>
                                        {menu_view(date_map.get(), set_selected_date)}
                                    </>
                                }.into_any()
                            } else {
                                menu_view(date_map.get(), set_selected_date)
                            }
                        }else{
                            let prev_date = entries[entries.len()-1].pub_date - chrono::Duration::days(7);
                            let next_date = entries[0].pub_date + chrono::Duration::days(1);
                            let today = Utc::now().date_naive();
                            view!{
                                <>
                                    <div class="flex justify-center mb-4">
                                        <A href=format!("/ui/photos/{}", prev_date.format("%y%m%d")) attr:class="btn btn-lg btn-accent">
                                            Previous Week
                                        </A>
                                    </div>
                                    {photo_list_view(entries)}
                                    {if next_date <= today {
                                        view! {
                                            <div class="flex justify-center mt-4">
                                                <A href=format!("/ui/photos/{}", next_date.format("%y%m%d")) attr:class="btn btn-lg btn-accent">
                                                    Next Week
                                                </A>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </>
                            }.into_any()
                        }
                    }
                }}
            </div>
        </>
    }
}