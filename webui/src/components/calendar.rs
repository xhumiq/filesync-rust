use leptos::prelude::*;
use chrono::{Datelike, NaiveDate, Utc};
use std::collections::HashMap;

#[component]
pub fn Calendar(
    available_dates: Option<HashMap<NaiveDate, usize>>,
    set_selected_date: WriteSignal<Option<NaiveDate>>
) -> impl IntoView {
    let today = Utc::now().date_naive();
    let default_date = available_dates.as_ref().and_then(|map| map.keys().max().cloned()).unwrap_or(today);
    let (current_date, set_current_date) = signal(default_date);

    let month_name = |month: u32| -> &'static str {
        match crate::get_current_language_code().as_str() {
            "zh" => match month {
                1 => "一月",
                2 => "二月",
                3 => "三月",
                4 => "四月",
                5 => "五月",
                6 => "六月",
                7 => "七月",
                8 => "八月",
                9 => "九月",
                10 => "十月",
                11 => "十一月",
                12 => "十二月",
                _ => "",
            },
            "fr" => match month {
                1 => "Janvier",
                2 => "Février",
                3 => "Mars",
                4 => "Avril",
                5 => "Mai",
                6 => "Juin",
                7 => "Juillet",
                8 => "Août",
                9 => "Septembre",
                10 => "Octobre",
                11 => "Novembre",
                12 => "Décembre",
                _ => "",
            },
            _ => match month {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "",
            },
        }
    };

    let days_in_month = move |year: i32, month: u32| -> u32 {
        let next_month = if month == 12 { 1 } else { month + 1 };
        let next_year = if month == 12 { year + 1 } else { year };
        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("Invalid date for first of next month");
        first_of_next.pred_opt().expect("No previous date").day()
    };

    let first_day_of_month = move |year: i32, month: u32| -> u32 {
        NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date for first of month").weekday().num_days_from_sunday()
    };

    view! {
        <div class="max-w-sm p-4 bg-white rounded-lg shadow-lg calendar">
            <div class="flex items-center justify-between mb-4 header">
                <button
                    class="btn btn-sm btn-outline"
                    on:click=move |_| {
                        let mut date = current_date.get();
                        if date.month() == 1 {
                            date = NaiveDate::from_ymd_opt(date.year() - 1, 12, 1).expect("Invalid date for previous month");
                        } else {
                            date = NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 1).expect("Invalid date for previous month");
                        }
                        set_current_date.set(date);
                    }
                >
                    "‹"
                </button>
                <h3 class="text-lg font-semibold">
                    {move || {
                        let date = current_date.get();
                        format!("{} {}", month_name(date.month()), date.year())
                    }}
                </h3>
                <button
                    class="btn btn-sm btn-outline"
                    on:click=move |_| {
                        let mut date = current_date.get();
                        if date.month() == 12 {
                            date = NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).expect("Invalid date for next month");
                        } else {
                            date = NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).expect("Invalid date for next month");
                        }
                        set_current_date.set(date);
                    }
                >
                    "›"
                </button>
            </div>

            <div class="grid grid-cols-7 gap-1 days-grid">
                // Day headers
                {
                    let days = match crate::get_current_language_code().as_str() {
                        "zh" => ["日", "一", "二", "三", "四", "五", "六"],
                        "fr" => ["Dim", "Lun", "Mar", "Mer", "Jeu", "Ven", "Sam"],
                        _ => ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
                    };
                    days.into_iter().map(|day| view! {
                        <div class="py-2 font-medium text-center text-gray-600 day-header">{day}</div>
                    }).collect_view()
                }

                // Empty cells for days before first day of month
                {move || {
                    let date = current_date.get();
                    let first_day = first_day_of_month(date.year(), date.month());
                    (0..first_day).map(|_| {
                        view! { <div class="empty-day"></div> }
                    }).collect_view()
                }}

                // Days of the month
                {move || {
                    let date = current_date.get();
                    let days = days_in_month(date.year(), date.month());
                    let today = Utc::now().date_naive();

                    (1..=days).map(|day| {
                        let day_date = NaiveDate::from_ymd_opt(date.year(), date.month(), day).expect("Invalid date for day");
                        let is_today = day_date == today;
                        let has_entries = available_dates.as_ref().map_or(false, |map| map.contains_key(&day_date));
                        let can_select = available_dates.as_ref().map_or(true, |map| map.contains_key(&day_date));
                        let class = if has_entries {
                            "day available bg-green-200 text-gray-800 rounded-full w-8 h-8 flex items-center justify-center cursor-pointer hover:bg-green-300"
                        } else if is_today {
                            "day today bg-gray-200 text-gray-800 rounded-full w-8 h-8 flex items-center justify-center cursor-pointer hover:bg-gray-300"
                        } else {
                            "day text-gray-400 rounded-full w-8 h-8 flex items-center justify-center"
                        };

                        view! {
                            <div
                                class=class
                                on:click=move |_| { if can_select { set_selected_date.set(Some(day_date)); } }
                            >
                                {day}
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}