leptos_i18n::load_locales!();
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use leptos_meta::*;
use leptos_router::{components::*, path};
use leptos_router::hooks::{use_navigate, use_location};
use serde_json;
use chrono::{DateTime, Utc, Local, FixedOffset};
use anyhow::{anyhow, Result};
use js_sys;

use crate::i18n::*;
use crate::pages::videos::VideoView;
use crate::pages::audio::AudioView;
use crate::pages::photos::PhotosView;
use crate::pages::home::Home;
use crate::pages::login::Login;
use crate::pages::custom::Custom;
use crate::pages::folder::Folder;
use crate::pages::not_found::NotFound;
use crate::components::private::Private;
use crate::models::auth::{AuthResponse};
use crate::app_state::{provide_app_state, use_folder};
// Modules
mod api;
mod components;
mod icons;
mod models;
mod pages;
mod langs;
mod app_state;
mod storage;

pub fn utc_to_local(utc_date_str: &str) -> DateTime<FixedOffset> {
    // Parse the RFC3339 string to DateTime<Utc>
    let dt_utc: DateTime<Utc> = match DateTime::parse_from_rfc3339(utc_date_str) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&FixedOffset::east_opt(0).unwrap()), // fallback
    };

    // Create JS Date from timestamp to get local offset
    let timestamp_ms = dt_utc.timestamp_millis() as f64;
    let js_date = js_sys::Date::new_0();
    js_date.set_time(timestamp_ms);

    // Get timezone offset in minutes (positive for west of UTC)
    let offset_minutes = js_date.get_timezone_offset() as i32;
    let offset_seconds = - (offset_minutes * 60); // FixedOffset east seconds

    let local_offset = FixedOffset::east_opt(offset_seconds).unwrap();

    dt_utc.with_timezone(&local_offset)
}

#[component]
fn PrivateHomeView() -> impl IntoView {
    view! { <Private><Home /></Private> }
}

// Private wrapper components
#[component]
fn PrivateVideoView() -> impl IntoView {
    view! { <Private><VideoView /></Private> }
}

#[component]
fn PrivateAudioView() -> impl IntoView {
    view! { <Private><AudioView /></Private> }
}

#[component]
fn PrivatePhotosView() -> impl IntoView {
    view! { <Private><PhotosView /></Private> }
}

#[component]
fn PrivateFolderView() -> impl IntoView {
    view! { <Private><Folder /></Private> }
}

#[component]
fn PrivateBrowseView() -> impl IntoView {
    view! { <Private><Custom /></Private> }
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {

    let git_sha = match option_env!("VERGEN_GIT_SHA") { Some(s) => s, None => "unknown" };
    let git_describe = match option_env!("VERGEN_GIT_DESCRIBE") { Some(s) => s, None => "unknown" };
    let git_commit_timestamp = match option_env!("VERGEN_GIT_COMMIT_TIMESTAMP") { Some(s) => s, None => "unknown" };
    let git_branch = match option_env!("VERGEN_GIT_BRANCH") { Some(s) => s, None => "unknown" };
    let git_commit_author_email = match option_env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL") { Some(s) => s, None => "unknown" };
    let git_commit_author_name = match option_env!("VERGEN_GIT_COMMIT_AUTHOR_NAME") { Some(s) => s, None => "unknown" };
    let git_commit_count = match option_env!("VERGEN_GIT_COMMIT_COUNT") { Some(s) => s, None => "unknown" };
    let git_commit_date = match option_env!("VERGEN_GIT_COMMIT_DATE") { Some(s) => s, None => "unknown" };
    let git_commit_message = match option_env!("VERGEN_GIT_COMMIT_MESSAGE") { Some(s) => s, None => "unknown" };
    let git_dirty = match option_env!("VERGEN_GIT_DIRTY") { Some(s) => s, None => "unknown" };

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_app_state();

    view! {
        <I18nContextProvider>
            <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

            // sets the document title
            <Title text="雅各家網站 - ACP GJCC - Zion Spiritual Flow" />

            // Set initial locale from localStorage or browser language
            {move || {
                let (i18n, _) = langs::get_locale();
            }}

            // injects metadata in the <head> of the page
            <Meta charset="UTF-8" />
            <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <Router>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=PrivateHomeView />
                    <Route path=path!("/account/login") view=Login />
                    <Route path=path!("/ui/videos/*path") view=PrivateVideoView />
                    <Route path=path!("/ui/audio/*path") view=PrivateAudioView />
                    <Route path=path!("/ui/docs/*path") view=PrivateFolderView />
                    <Route path=path!("/ui/photos/*path") view=PrivatePhotosView />
                    <Route path=path!("/ui/hymns/*path") view=PrivateFolderView />
                    <Route path=path!("/browse/*path") view=PrivateBrowseView />
                    <Route path=path!("/files/*path") view=PrivateFolderView />
                </Routes>
            </Router>
        </I18nContextProvider>
        <script>
            window.buildInfo={serde_json::to_string_pretty(&serde_json::json!({
                "SHA": git_sha,
                "DESCRIBE": git_describe,
                "COMMIT_TIMESTAMP": git_commit_timestamp,
                "BRANCH": git_branch,
                "COMMIT_AUTHOR_EMAIL": git_commit_author_email,
                "COMMIT_AUTHOR_NAME": git_commit_author_name,
                "COMMIT_COUNT": git_commit_count,
                "COMMIT_DATE": git_commit_date,
                "COMMIT_MESSAGE": git_commit_message,
                "DIRTY": git_dirty
            })).unwrap()}
        </script>
    }
}
