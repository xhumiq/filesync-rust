use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u32,
    refresh_expires_in: u32,
    refresh_token: Option<String>,
    token_type: String,
    #[serde(rename = "not-before-policy")]
    not_before_policy: u32,
    session_state: Option<String>,
    scope: Option<String>,
    id_token: Option<String>,
}

async fn authenticate_user(
    keycloak_url: &str,
    realm: &str,
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let token_url = format!("{}/realms/{}/protocol/openid-connect/token", keycloak_url, realm);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("grant_type", "password");
    params.insert("username", username);
    params.insert("password", password);
    params.insert("scope", "openid");

    let response = client
        .post(&token_url)
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let token: TokenResponse = response.json().await?;
        Ok(token)
    } else {
        let error_text = response.text().await?;
        Err(format!("Authentication failed: {}", error_text).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage - replace with actual values
    let keycloak_url = "https://auth.kefacp.com:8543";
    let realm = "chat-acp";
    let client_id = "dgTHvreKNMuCJe1LMVYIEE1grjKR";
    let client_secret = "FqVpd23rIgWXbzzl6rDQJ5d7VTcc3CwK";
    let username = "mona";
    let password = "z4ACCTKGe9AB";

    match authenticate_user(keycloak_url, realm, client_id, client_secret, username, password).await {
        Ok(token) => {
            println!("Authentication successful!");
            println!("Access Token: {}", token.access_token);
            println!("Token Type: {}", token.token_type);
            println!("Expires In: {} seconds", token.expires_in);
            if let Some(refresh) = token.refresh_token {
                println!("Refresh Token: {}", refresh);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
