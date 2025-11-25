use gloo_net::http::Request;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::models::channel::Channel;

pub fn get_api_file_listing_url() -> String {
    std::env::var("API_FILE_LISTING_URL")
        .unwrap_or_else(|_| "/fs/v1".to_string())
}

pub fn get_jwt_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("jwt_token").ok().flatten())
}

pub async fn fetch_files(path: String) -> AnyhowResult<Channel> {
    let url = format!(
        "{}/{}",
        get_api_file_listing_url(),
        path.trim_start_matches('/')
    );

    let jwt = get_jwt_token().ok_or_else(|| anyhow!("No JWT token found"))?;

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {jwt}"))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e:?}"))?;

    if !resp.ok() {
        if resp.status() == 401 {
            // Redirect to login page on 401 Unauthorized
            if let Some(window) = web_sys::window() {
                if let Some(_location) = window.location().href().ok() {
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

pub fn format_size(bytes: u64) -> String {
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


// fn list_weeks_in_range(start_date: NaiveDate, end_date: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
//     let mut weeks = Vec::new();
//     let mut current = start_date;

//     while current <= end_date {
//         // Find the Saturday of the current week (or end_date if earlier)
//         let days_to_saturday = (6 - current.weekday().num_days_from_sunday()) as i64;
//         let week_end = current + chrono::Duration::days(days_to_saturday);
//         let actual_end = if week_end > end_date { end_date } else { week_end };

//         weeks.push((current, actual_end));

//         // If we've reached the end date, we're done
//         if actual_end >= end_date {
//             break;
//         }

//         // Move to the next Sunday
//         current = actual_end + chrono::Duration::days(1);
//         // If current is not Sunday, find the next Sunday
//         if current.weekday() != Weekday::Sun {
//             let days_to_sunday = (7 - current.weekday().num_days_from_sunday()) % 7;
//             current = current + chrono::Duration::days(days_to_sunday as i64);
//         }
//     }

//     weeks.reverse();
//     weeks
// }
