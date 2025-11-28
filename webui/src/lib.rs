leptos_i18n::load_locales!();
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use serde_json;
use crate::i18n::*; // `i18n` module created by the macro above
// Modules
mod api;
mod components;
mod icons;
mod models;
mod pages;

// Top-Level pages
use crate::pages::folder::Folder;
use crate::pages::videos::VideoView;
use crate::pages::audio::AudioView;
use crate::pages::photos::PhotosView;
use crate::pages::home::Home;
use crate::pages::login::Login;
use crate::pages::not_found::NotFound;

/// Get the current language code from localStorage or browser language
pub fn get_current_language_code() -> String {
    if let Some(window) = web_sys::window() {
        // First check localStorage
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(locale_str)) = storage.get_item("locale") {
                if locale_str.starts_with("zh") {
                    return "zh".to_string();
                } else if locale_str.starts_with("fr") {
                    return "fr".to_string();
                }
                return "en".to_string();
            }
        }

        // Fall back to browser language
        match window.navigator().language() {
            Some(lang) => {
                if lang.starts_with("zh") {
                    return "zh".to_string();
                } else if lang.starts_with("fr") {
                    return "fr".to_string();
                }
            }
            None => {}
        }
    }

    // Default to English
    "en".to_string()
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

    view! {
        <I18nContextProvider>
            <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

            // sets the document title
            <Title text="雅各家網站 - ACP GJCC - Zion Spiritual Flow" />

            // Set initial locale from localStorage or browser language
            {move || {
                let i18n = use_i18n();
                if let Some(window) = web_sys::window() {
                    // First check localStorage
                    let mut locale = if let Ok(Some(storage)) = window.local_storage() {
                        if let Ok(Some(locale_str)) = storage.get_item("locale") {
                            match locale_str.as_str() {
                                "zh" => Some(Locale::zh),
                                "en" => Some(Locale::en),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // If not in localStorage, check browser language
                    if locale.is_none() {
                        match window.navigator().language() {
                            Some(lang) => {
                                if lang.starts_with("zh") {
                                    locale = Some(Locale::zh);
                                } else if lang.starts_with("fr") {
                                    locale = Some(Locale::fr);
                                } else {
                                    locale = Some(Locale::en);
                                }
                            }
                            None => {
                                locale = Some(Locale::en);
                            }
                        }
                    }

                    // Set the locale
                    if let Some(loc) = locale {
                        i18n.set_locale(loc);
                    }
                }
            }}

            // injects metadata in the <head> of the page
            <Meta charset="UTF-8" />
            <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <Router>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/account/login") view=Login />
                    <Route path=path!("/ui/videos/*path") view=VideoView />
                    <Route path=path!("/ui/audio/*path") view=AudioView />
                    <Route path=path!("/ui/docs/*path") view=Folder />
                    <Route path=path!("/ui/photos/*path") view=PhotosView />
                    <Route path=path!("/ui/hymns/*path") view=Folder />
                    <Route path=path!("/files/*path") view=Folder />
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
