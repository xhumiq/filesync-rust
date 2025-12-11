use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use crate::i18n::{use_i18n, t_string};
use crate::api::*;
use crate::app_state::*;
use crate::utc_to_local;
use crate::storage::store_auth;

#[component]
pub fn LoginNew() -> impl IntoView {
    let i18n = use_i18n();
    let app_state = use_app_state();
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (email_focused, set_email_focused) = signal(false);
    let (password_focused, set_password_focused) = signal(false);

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
        let state = app_state.clone();
        ev.prevent_default();
        set_error_message.set(String::new());

        let email_val = email.get();
        let password_val = password.get();

        // Validation
        if email_val.len() < 3 || email_val.len() > 24 {
            set_error_message.set(t_string!(i18n, username_validation).to_string());
            return;
        }

        if password_val.len() < 5 {
            set_error_message.set(t_string!(i18n, password_validation).to_string());
            return;
        }

        // Make HTTP request
        spawn_local(async move {
            match login(i18n, &email_val, &password_val).await {
                Ok(login_resp) => {
                    state.auth.set(Some(login_resp.clone()));
                    if let Err(e) = store_auth(&login_resp) {
                        leptos::logging::error!("Failed to store auth: {:?}", e);
                    }
                    if let Some(refresh) = login_resp.refresh_token.clone() {
                        let local_expires = utc_to_local(&login_resp.expires_at);
                        schedule_refresh_token(&state, refresh, local_expires);
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

    view! {
        <div class="flex items-center justify-center min-h-screen" style="background-color: #f5f5f5;">
            <style>
                {r#"
                .floating-label-container {
                    position: relative;
                    margin-bottom: 24px;
                }
                
                .floating-input {
                    width: 100%;
                    padding: 16px 12px 8px 12px;
                    font-size: 16px;
                    border: 2px solid #e0e0e0;
                    border-radius: 8px;
                    outline: none;
                    background: white;
                    transition: border-color 0.2s;
                }
                
                .floating-input:focus {
                    border-color: #4A90E2;
                }
                
                .floating-input.has-value {
                    border-color: #4A90E2;
                }
                
                .floating-label {
                    position: absolute;
                    left: 12px;
                    top: 16px;
                    font-size: 16px;
                    color: #999;
                    pointer-events: none;
                    transition: all 0.2s ease;
                    background: white;
                    padding: 0 4px;
                }
                
                .floating-input:focus ~ .floating-label,
                .floating-input.has-value ~ .floating-label {
                    top: -8px;
                    font-size: 14px;
                    color: #4A90E2;
                    font-weight: 500;
                }
                
                .password-container {
                    position: relative;
                }
                
                .show-password-btn {
                    position: absolute;
                    right: 12px;
                    top: 50%;
                    transform: translateY(-50%);
                    background: #f0f0f0;
                    border: none;
                    padding: 6px 12px;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 14px;
                    color: #666;
                    transition: background-color 0.2s;
                }
                
                .show-password-btn:hover {
                    background: #e0e0e0;
                }
                
                .sign-in-btn {
                    width: 100%;
                    padding: 14px;
                    background: #4A90E2;
                    color: white;
                    border: none;
                    border-radius: 8px;
                    font-size: 16px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: background-color 0.2s;
                }
                
                .sign-in-btn:hover {
                    background: #3A7BC8;
                }
                
                .login-card {
                    background: white;
                    border-radius: 12px;
                    padding: 48px 40px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
                    width: 100%;
                    max-width: 460px;
                }
                
                .login-title {
                    font-size: 32px;
                    font-weight: 700;
                    color: #2c3e50;
                    text-align: center;
                    margin-bottom: 40px;
                }
                
                .error-message {
                    background: #fee;
                    color: #c33;
                    padding: 12px;
                    border-radius: 6px;
                    margin-bottom: 20px;
                    font-size: 14px;
                }
                "#}
            </style>
            
            <div class="login-card">
                <h1 class="login-title">"GJCC File Server"</h1>
                
                <form on:submit=on_submit>
                    {move || {
                        let error = error_message.get();
                        if !error.is_empty() {
                            view! {
                                <div class="error-message">
                                    {error}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                    
                    <div class="floating-label-container">
                        <input
                            type="text"
                            class=move || if !email.get().is_empty() || email_focused.get() { "floating-input has-value" } else { "floating-input" }
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            on:focus=move |_| set_email_focused.set(true)
                            on:blur=move |_| set_email_focused.set(false)
                            required
                        />
                        <label class="floating-label">"User Name"</label>
                    </div>
                    
                    <div class="floating-label-container">
                        <div class="password-container">
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                class=move || if !password.get().is_empty() || password_focused.get() { "floating-input has-value" } else { "floating-input" }
                                style="padding-right: 70px;"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                on:focus=move |_| set_password_focused.set(true)
                                on:blur=move |_| set_password_focused.set(false)
                                required
                            />
                            <button
                                type="button"
                                class="show-password-btn"
                                on:click=move |_| set_show_password.update(|v| *v = !*v)
                            >
                                {move || if show_password.get() { "Hide" } else { "Show" }}
                            </button>
                            <label class="floating-label">"Password"</label>
                        </div>
                    </div>
                    
                    <button type="submit" class="sign-in-btn">
                        "Sign in"
                    </button>
                </form>
            </div>
        </div>
    }
}