use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::i18n::{use_i18n, t, t_string};
use crate::api::*;
use crate::app_state::*;
use crate::utc_to_local;
use crate::storage::store_auth;

#[component]
pub fn Login() -> impl IntoView {
    let i18n = use_i18n();
    let app_state = use_app_state();
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (remember_me, set_remember_me) = signal(false);
    let (error_message, set_error_message) = signal(String::new());

    // Load username from cookie on mount
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(html_doc) = window.document().expect("No document").dyn_into::<web_sys::HtmlDocument>() {
                let cookies: String = html_doc.cookie().unwrap_or_default();
                for cookie in cookies.split(';') {
                    let cookie: &str = cookie.trim();
                    if cookie.starts_with("username=") {
                        if let Some(value) = cookie.strip_prefix("username=") {
                            set_email.set(urlencoding::decode(value).unwrap_or_default().to_string());
                        }
                    }
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(String::new());

        let email_val = email.get();
        let password_val = password.get();
        let remember = remember_me.get();

        // Validation
        if email_val.len() < 3 || email_val.len() > 24 {
            set_error_message.set(t_string!(i18n, username_validation).to_string());
            return;
        }

        if password_val.len() < 5 {
            set_error_message.set(t_string!(i18n, password_validation).to_string());
            return;
        }

        // Set cookie if remember me is checked
        if remember {
            if let Some(window) = web_sys::window() {
                if let Ok(html_doc) = window.document().expect("No document").dyn_into::<web_sys::HtmlDocument>() {
                    let expires = js_sys::Date::now() + (30.0 * 24.0 * 60.0 * 60.0 * 1000.0); // 30 days
                    let date = js_sys::Date::new_0();
                    date.set_time(expires);
                    let cookie_value = format!("username={}; expires={}", urlencoding::encode(&email_val), date.to_utc_string().as_string().unwrap_or_default());
                    let _: Result<(), wasm_bindgen::JsValue> = html_doc.set_cookie(&cookie_value);
                }
            }
        }

        // Make HTTP request
        spawn_local(async move {
            match login(i18n, &email_val, &password_val).await {
                Ok(login_resp) => {
                    app_state.auth.set(Some(login_resp.clone()));
                    if let Err(e) = store_auth(&login_resp) {
                        leptos::logging::error!("Failed to store auth: {:?}", e);
                    }
                    if let Some(refresh) = login_resp.refresh_token.clone() {
                        let local_expires = utc_to_local(&login_resp.expires_at);
                        schedule_refresh_token(refresh, local_expires);
                    }
                    // Redirect to home page
                    if let Some(window) = web_sys::window() {
                        let mut location = "/".to_string();
                        if let Some(win_location) = window.location().href().ok() {
                            location = win_location.clone();
                        }
                        if location.ends_with("/login") {
                            location = "/".to_string();
                        }
                        leptos::logging::log!("Redirect to {}", &location);
                        let _ = window.location().set_href(&location);
                    }
                }
                Err(e) => {
                    set_error_message.set(e.to_string());
                }
            }
        });
    };

    let on_forgot_password = move |_| {
        // Handle forgot password logic here
        leptos::logging::log!("Forgot password clicked");
    };

    view! {
        <div class="flex items-center justify-center min-h-screen bg-base-200">
            <div class="w-full max-w-md shadow-xl card bg-base-100">
                <div class="card-body">
                    <h2 class="mb-2 text-3xl text-center card-title">{t!(i18n, login_title)}</h2>

                    <form on:submit=on_submit>
                        <div class="form-control">
                            <label class="mb-1 label">
                                <span class="label-text">{t!(i18n, username)}</span>
                            </label>
                            <input
                                type="text"
                                placeholder=move || t_string!(i18n, username_placeholder)
                                class="input input-bordered"
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div class="form-control">
                            <label class="label">
                                <span class="label-text">{t!(i18n, password)}</span>
                            </label>
                            <input
                                type="password"
                                placeholder=move || t_string!(i18n, password_placeholder)
                                class="input input-bordered"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div class="form-control">
                            <label class="cursor-pointer label">
                                <span class="label-text">{t!(i18n, remember_me)}</span>
                                <input
                                    type="checkbox"
                                    class="checkbox"
                                    prop:checked=remember_me
                                    on:change=move |ev| set_remember_me.set(event_target_checked(&ev))
                                />
                            </label>
                        </div>

                        {move || {
                            let error = error_message.get();
                            if !error.is_empty() {
                                view! {
                                    <div class="alert mt-4 !bg-red-900 !text-white !border-red-900">
                                        <span>{error}</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}

                        <div class="mt-6 form-control">
                            <button type="submit" class="btn btn-primary">{t!(i18n, login)}</button>
                        </div>
                    </form>

                    <div class="divider">{t!(i18n, or)}</div>

                    <div class="text-center">
                        <button
                            class="btn btn-link"
                            on:click=on_forgot_password
                        >
                            {t!(i18n, forgot_password)}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
