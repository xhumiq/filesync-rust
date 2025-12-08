use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_location};
use crate::models::auth::is_token_valid;
use crate::pages::folder::Folder;
use crate::app_state::{provide_app_state, use_folder, use_app_state};
use crate::storage::get_jwt_token;

#[component]
pub fn Private(children: Children) -> AnyView {
    let navigate = use_navigate();
    let folder = use_folder();
    let jwt = get_jwt_token();
    let logged_in = if let Some(jwt) = jwt {
        is_token_valid(&jwt)
    } else { false };
    Effect::new(move |_| {
        if !logged_in {
            let nav = navigate.clone();
            nav("/account/login", Default::default());
            return
        }
        let location = use_location();
        let pathname = location.pathname.get();
        if folder.get().is_some() && !pathname.starts_with("/browse/") {
            let nav = navigate.clone();
            nav("/browse", Default::default());
            return
        }
    });

    if logged_in {
        children()
    } else {
        view! { <div></div> }.into_any()
    }
}