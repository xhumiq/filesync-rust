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
use crate::models::channel::{Channel, FolderShare};
use crate::models::auth::{AuthResponse, Claims};
use crate::api::refresh_token_request;
use crate::storage::{get_auth_from_store, store_auth, clear_tokens};
use crate::{utc_to_local};

// Define your app's shared state
#[derive(Clone, Debug)]
pub struct AppState {
  pub domain: String,
  pub auth: RwSignal<Option<AuthResponse>>,
}

pub fn provide_app_state() {
   let domain = web_sys::window()
     .and_then(|w| w.location().host().ok())
     .unwrap_or_else(|| "localhost".to_string());
   let auth = RwSignal::new(None);
   provide_context(AppState {
     domain,
     auth
   });
  let auth = get_auth_from_store();
  if let Some(auth) = auth {
    match set_auth_response(Some(auth)){
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

pub fn set_auth_response(response: Option<AuthResponse>) -> Result<Option<DateTime<FixedOffset>>>{
   let state = use_context::<AppState>().expect("AppState to be provided");
   state.auth.set(response.clone());
   match response {
     Some(resp) => {
       let local_expires = utc_to_local(&resp.expires_at);
       let local_refresh_expires = utc_to_local(&resp.refresh_expires_at);
       leptos::logging::log!("Token expires at: {} (local)", local_expires);
       leptos::logging::log!("Refresh token expires at: {} (local)", local_refresh_expires);
       store_auth(&resp)?;
       set_auth_cookie(&state.domain, &resp)?;
       return Ok(Some(local_expires));
     },
     None => {
       clear_tokens();
       return Ok(None);
     }
   }
 }

pub fn set_auth_cookie(domain: &str, response: &AuthResponse) -> Result<()>{
  let expires_timestamp = DateTime::parse_from_rfc3339(&response.expires_at)
    .map(|dt| dt.with_timezone(&Utc).timestamp())
    .unwrap_or(0);

  let options = UseCookieOptions::default()
    .expires(expires_timestamp)
    .domain(domain)
    .path("/")
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Strict);

  let (auth_cookie, set_auth_cookie) = use_cookie_with_options::<String, FromToStringCodec>(
    "auth",
    options
  );

  set_auth_cookie.set(Some(response.jwt_token.clone()));
  Ok(())
}

pub fn schedule_refresh_token(refresh_token: String, expires_at: DateTime<FixedOffset>) {
  leptos::logging::log!("Schedule Refresh Token Request");
  if let Some(window) = web_sys::window() {
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
      let refresh_token = refresh_token.clone();
      spawn_local(async move {
        match refresh_token_request(refresh_token).await{
          Ok(resp) => {
            set_auth_response(Some(resp)).unwrap();
          },
          Err(e) => {
            leptos::logging::error!("Failed to refresh token: {}", e);
          }
        }
      });
    }) as Box<dyn FnMut()>);

    // Calculate delay until 5 seconds before expiry
    let now_ms = js_sys::Date::now() as i64;
    let expires_ms = expires_at.timestamp_millis();
    let mut delay_ms = expires_ms - now_ms; // 5 seconds before

    if delay_ms > 0 {
      delay_ms = delay_ms / 2000;
      leptos::logging::log!("Scheduling token refresh in {} seconds", delay_ms);
      let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        delay_ms as i32,
      );
      closure.forget();
    } else {
      // Already expired or very close, don't schedule
      leptos::logging::log!("Token already expired or expiring soon, not scheduling refresh");
    }
  }
}
