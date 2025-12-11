use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::{use_cookie, use_cookie_with_options, UseCookieOptions, SameSite};
use codee::string::FromToStringCodec;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, Utc};
use serde_json;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use url::Url;
use crate::models::channel::{Channel, FolderShare};
use crate::models::auth::{AuthResponse, Claims};
use crate::api::{refresh_token_request, get_api_file_listing_url};
use crate::storage::{get_auth_from_store, store_auth, clear_tokens};
use crate::{utc_to_local};

// Define your app's shared state
#[derive(Clone, Debug)]
pub struct AppState {
  pub domain: String,
  pub auth: RwSignal<Option<AuthResponse>>,
  pub scheduled_refresh: RwSignal<Option<i32>>,
}

pub fn provide_app_state() {
   let api_url = get_api_file_listing_url();
   let domain = if let Ok(url) = Url::parse(&api_url) {
       url.host_str().unwrap_or("localhost").to_string()
   } else {
       web_sys::window()
         .and_then(|w| w.location().host().ok())
         .unwrap_or_else(|| "localhost".to_string())
   };
   let auth = RwSignal::new(None);
   let scheduled_refresh = RwSignal::new(None);
   let state = AppState {
     domain,
     auth,
     scheduled_refresh
   };
   provide_context(state.clone());
  let auth = get_auth_from_store();
  if let Some(auth) = auth {
    match set_auth_response(&state, Some(auth)){
      Ok(_) => {},
      Err(e) => {
        leptos::logging::error!("Failed to set auth: {e:?}");
      }
    }
  }else{
    leptos::logging::log!("Provide AppState no auth");
  }
}
pub fn use_app_state() -> AppState {
  use_context::<AppState>().expect("AppState to be provided")
}

pub fn use_claims() -> Option<Claims> {
  let state = use_context::<AppState>().expect("AppState to be provided");
  match state.auth.get(){
    Some(auth) => Some(auth.claims.clone()),
    None => None
  }
}

pub fn use_folder() -> Memo<Option<FolderShare>> {
   let state = use_context::<AppState>().expect("AppState to be provided");
   Memo::new(move |_| {
     match state.auth.get() {
       Some(auth) => auth.folder.clone(),
       None => None
     }
   })
 }

pub fn set_auth_response(state: &AppState, response: Option<AuthResponse>) -> Result<Option<DateTime<FixedOffset>>>{
  state.auth.set(response.clone());
  match response {
    Some(resp) => {
      store_auth(&resp)?;
      set_cookie("jwt_token", &resp.jwt_token, 0);
      let local_expires = utc_to_local(&resp.expires_at);
      if let Some(refresh) = resp.refresh_token{
        schedule_refresh_token(state, refresh, local_expires);
      }
      return Ok(Some(local_expires));
    },
    None => {
      clear_tokens();
      return Ok(None);
    }
  }
}

fn set_cookie(name: &str, value: &str, days: i32) {
    let document = document(); // This is web_sys::Document
    let val = js_sys::encode_uri_component(value);

    let mut cookie = format!("{name}={val}");

    if days != 0 {
        use wasm_bindgen::JsValue;
        let window = window();
        let location = window.location();
        let hostname = location.hostname().unwrap_or_default();

        // Basic expiration
        if days > 0 {
            let date = js_sys::Date::new_0();
            date.set_time(date.get_time() + (days as f64 * 24.0 * 60.0 * 60.0 * 1000.0));
            cookie.push_str(&format!("; expires={}", date.to_utc_string()));
        }

        // Recommended security flags in 2025
        cookie.push_str("; path=/");
        cookie.push_str(&format!("; Domain={}; HttpOnly", hostname));
        cookie.push_str("; Secure");                    // only sent over HTTPS
        cookie.push_str("; SameSite=Lax");           // or Lax if you need cross-site
        // cookie.push_str("; HttpOnly"); // ← you CANNOT set HttpOnly from JavaScript!
    }

    document
        .unchecked_ref::<web_sys::HtmlDocument>()
        .set_cookie(&cookie)
        .expect("setting cookie failed");
}

pub fn set_auth_cookie(domain: &str, response: &AuthResponse) -> Result<()>{
  let expires_timestamp = DateTime::parse_from_rfc3339(&response.expires_at)
    .map(|dt| dt.with_timezone(&Utc).timestamp())
    .unwrap_or(0);

  let current_timestamp = Utc::now().timestamp();
  let seconds_until_expiry = expires_timestamp - current_timestamp;
  leptos::logging::log!("{} Expires in {} seconds", domain, seconds_until_expiry);

  if seconds_until_expiry <= 0 {
    leptos::logging::log!("Token already expired or expiring, not setting cookie");
    return Ok(());
  }

  let secure = domain != "localhost:3030";

  let mut options = UseCookieOptions::default()
    .expires(expires_timestamp)
    .path("/")
    .secure(secure)
    .same_site(SameSite::Lax)
    .http_only(false);

  if domain != "localhost" {
    options = options.domain(domain);
  }

  let (auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "auth",
    options
  );

  set_auth_cookie.set(Some(response.jwt_token.clone()));
  set_cookie("jwt_token", &response.jwt_token, 0);
  Ok(())
}

pub fn logout(state: &AppState) {
    // Clear auth signal
    state.auth.set(None);
    
    // Clear jwt_token cookie using document object
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        let cookie = "jwt_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
        document
            .unchecked_ref::<web_sys::HtmlDocument>()
            .set_cookie(cookie)
            .ok();
    }
    
    // Clear tokens in local storage
    clear_tokens();
    
    // Clear refresh window time handle
    if let Some(handle) = state.scheduled_refresh.get() {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(handle);
        }
    }
    state.scheduled_refresh.set(None);
}



pub fn schedule_refresh_token(state: &AppState, refresh_token: String, expires_at: DateTime<FixedOffset>) {
  if let Some(window) = web_sys::window() {
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
      let refresh_token = refresh_token.clone();
      let state = state.clone();
      spawn_local(async move {
        match refresh_token_request(refresh_token).await{
          Ok(resp) => {
            set_auth_response(&state, Some(resp)).unwrap();
          },
          Err(e) => {
            leptos::logging::error!("Failed to refresh token: {}", e);
            if let Some(window) = web_sys::window() {
                if let Some(_location) = window.location().href().ok() {
                    let _ = window.location().set_href("/account/login");
                }
            }
          }
        }
      });
    }) as Box<dyn FnMut()>);

    // Calculate delay until 5 seconds before expiry
    let now_ms = js_sys::Date::now() as i64;
    let expires_ms = expires_at.timestamp_millis();
    let mut delay_ms = expires_ms - now_ms; // 5 seconds before

    if delay_ms > 15000 {
      delay_ms -= 10000;
    }else{
      delay_ms = 200;
    }
    leptos::logging::log!("Scheduling token refresh in {} seconds", delay_ms/1000);
    let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(
      closure.as_ref().unchecked_ref(),
      delay_ms as i32,
    );
    match handle {
      Ok(handle) => {
        state.scheduled_refresh.set(Some(handle));
      }
      Err(e) => {
        let error_msg = format!("{:?}", e);
        leptos::logging::error!("Failed to schedule token refresh: {}", error_msg);
      }
    }
    closure.forget();
  }
}
