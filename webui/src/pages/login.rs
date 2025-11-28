use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use gloo_net::http::Request;
use serde::Deserialize;
use crate::i18n::{use_i18n, t, t_string};

fn get_api_login_url() -> String {
    match option_env!("API_LOGIN_URL") { Some(s) => s.to_string(), None => "/auth/v1/login".to_string() }
}

fn get_api_refresh_token_url() -> String {
    match option_env!("API_REFRESH_TOKEN_URL") { Some(s) => s.to_string(), None => "/auth/v1/refresh".to_string() }
}

#[derive(Deserialize)]
struct LoginResponse {
    jwt_token: String,
    refresh_token: String,
    expires_at: String,
    refresh_expires_at: String,
}

fn store_tokens(jwt: &str, refresh: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("jwt_token", jwt);
            let _ = storage.set_item("refresh_token", refresh);
        }
    }
}

fn utc_to_local(utc_date_str: &str) -> String {
    // For now, just return the UTC string - in a real implementation,
    // you'd use JavaScript's Date API to convert to local timezone
    utc_date_str.to_string()
}

fn schedule_refresh_token(refresh_token: String, _refresh_expires_at: String) {
    // For now, schedule a simple timeout - in a real implementation,
    // you'd calculate the exact time until 5 seconds before expiry
    if let Some(window) = web_sys::window() {
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            let refresh_token = refresh_token.clone();
            spawn_local(async move {
                refresh_token_request(refresh_token).await;
            });
        }) as Box<dyn FnMut()>);

        // Schedule refresh in 10 seconds for testing (should be calculated properly)
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            10000, // 10 seconds for testing
        );
        closure.forget();
    }
}

async fn refresh_token_request(refresh_token: String) {
    let body = serde_json::json!({
        "refresh_token": refresh_token,
    });

    match Request::post(&get_api_refresh_token_url())
        .header("Content-Type", "application/json")
        .json(&body)
    {
        Ok(request) => {
            match request.send().await {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<LoginResponse>().await {
                            Ok(login_resp) => {
                                leptos::logging::log!("Token refresh successful");
                                store_tokens(&login_resp.jwt_token, &login_resp.refresh_token);

                                // Print expiration times in local timezone
                                let local_expires = utc_to_local(&login_resp.expires_at);
                                let local_refresh_expires = utc_to_local(&login_resp.refresh_expires_at);
                                leptos::logging::log!("New token expires at: {} (local)", local_expires);
                                leptos::logging::log!("New refresh token expires at: {} (local)", local_refresh_expires);

                                // Schedule next refresh
                                schedule_refresh_token(login_resp.refresh_token, login_resp.refresh_expires_at);
                            }
                            Err(e) => {
                                leptos::logging::error!("Failed to parse refresh response: {:?}", e);
                            }
                        }
                    } else {
                        leptos::logging::error!("Token refresh failed with status: {}", resp.status());
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Network error during token refresh: {:?}", e);
                }
            }
        }
        Err(e) => {
            leptos::logging::error!("Failed to create refresh request: {:?}", e);
        }
    }
}

#[component]
pub fn Login() -> impl IntoView {
    let i18n = use_i18n();
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
            let body = serde_json::json!({
                "username": email_val,
                "password": password_val,
            });

            match Request::post(&get_api_login_url())
                .header("Content-Type", "application/json")
                .json(&body)
            {
                Ok(request) => {
                    match request.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<LoginResponse>().await {
                                    Ok(login_resp) => {
                                        leptos::logging::log!("Login successful: {}", email_val);

                                        // Print expiration times in local timezone
                                        let local_expires = utc_to_local(&login_resp.expires_at);
                                        let local_refresh_expires = utc_to_local(&login_resp.refresh_expires_at);
                                        leptos::logging::log!("Token expires at: {} (local)", local_expires);
                                        leptos::logging::log!("Refresh token expires at: {} (local)", local_refresh_expires);

                                        store_tokens(&login_resp.jwt_token, &login_resp.refresh_token);

                                        // Schedule automatic token refresh
                                        schedule_refresh_token(login_resp.refresh_token.clone(), login_resp.refresh_expires_at.clone());

                                        // Redirect to home page
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.location().set_href("/ui/videos/today");
                                        }
                                    }
                                    Err(e) => {
                                        leptos::logging::error!("Failed to parse response: {:?}", e);
                                        set_error_message.set(t_string!(i18n, invalid_response).to_string());
                                    }
                                }
                            } else {
                                set_error_message.set(t_string!(i18n, invalid_credentials).to_string());
                            }
                        }
                        Err(e) => {
                            leptos::logging::error!("Network error: {:?}", e);
                            set_error_message.set(t_string!(i18n, network_error).to_string());
                        }
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Failed to create request: {:?}", e);
                    set_error_message.set(t_string!(i18n, request_error).to_string());
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
