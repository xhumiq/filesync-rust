use leptos::*;
use anyhow::{anyhow, Result};
use crate::models::auth::{AuthResponse};
use serde_json;

pub fn store_auth(resp: &AuthResponse) -> Result<()>{
  if let Some(window) = web_sys::window() {
    if let Ok(Some(storage)) = window.local_storage() {
      let auth_json = serde_json::to_string(resp).map_err(|e| anyhow!("Failed to serialize auth response: {e:?}"))?;
      match storage.set_item("auth", &auth_json) {
        Ok(_) => {},
        Err(e) => {
          leptos::logging::error!("Failed to store auth: {:?}", e);
          return Err(anyhow!("Failed to store auth: {e:?}"));
        }
      }
      match storage.set_item("jwt_token", &resp.jwt_token) {
        Ok(_) => {},
        Err(e) => {
          leptos::logging::error!("Failed to store jwt_token: {:?}", e);
          return Err(anyhow!("Failed to store jwt_token: {e:?}"));
        }
      }
    }
  }
  return Ok(());
}

pub fn get_auth_from_store() -> Option<AuthResponse> {
  web_sys::window()
    .and_then(|w| w.local_storage().ok().flatten())
    .and_then(|s| s.get_item("auth").ok().flatten())
    .and_then(|auth_json| serde_json::from_str::<AuthResponse>(&auth_json).ok())
}

pub fn get_jwt_token() -> Option<String> {
  web_sys::window()
    .and_then(|w| w.local_storage().ok().flatten())
    .and_then(|s| s.get_item("jwt_token").ok().flatten())
}

pub fn clear_tokens() -> Result<()>{
  leptos::logging::log!("Clearing tokens");
  if let Some(window) = web_sys::window() {
    if let Ok(Some(storage)) = window.local_storage() {
      match storage.remove_item("auth") {
        Ok(_) => {},
        Err(e) => {
          leptos::logging::error!("Failed to clear auth: {:?}", e);
          return Err(anyhow!("Failed to clear auth: {e:?}"));
        }
      }
      match storage.remove_item("jwt_token") {
        Ok(_) => {},
        Err(e) => {
          leptos::logging::error!("Failed to clear jwt_token: {:?}", e);
          return Err(anyhow!("Failed to clear jwt_token: {e:?}"));
        }
      }
    }
  }
  return Ok(());
}
