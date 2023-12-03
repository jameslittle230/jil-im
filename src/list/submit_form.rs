use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Form,
};

use serde::{Deserialize, Serialize};
use strum::Display;
use tower_sessions::Session;

use crate::{
    api_client::ApiClient,
    create::FormValues,
    state::State,
    util::flash::{flash, flash_error_alert, flash_info_alert, flash_success_alert, FlashType},
};

#[derive(Debug, Display, Serialize, Deserialize)]
pub(crate) enum FormAction {
    #[serde(rename = "edit")]
    Edit,

    #[serde(rename = "delete")]
    Delete,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FormData {
    action: FormAction,
    password: Option<String>,
    shortname: String,
}

pub(crate) async fn submit_form(
    mut session: Session,
    Extension(state): Extension<Arc<Mutex<State>>>,
    Form(mut form): Form<FormData>,
) -> impl IntoResponse {
    match form.action {
        FormAction::Edit => {
            let state = state.lock().await;
            let link = state.links.get(&form.shortname).unwrap();
            flash_info_alert(
                format!(
                    "Editing link for {}/{}",
                    std::env::var("BASE_URL").unwrap(),
                    form.shortname
                ),
                &mut session,
            );

            flash(
                FlashType::CreateFormUserValues,
                FormValues {
                    shortname: link.shortname.clone(),
                    longurl: link.longurl.clone(),
                    shortname_is_disabled: true,
                },
                &mut session,
            );
            (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
        }
        FormAction::Delete => {
            let mut state = state.lock().await;
            if !bcrypt::verify(form.password.unwrap_or_default(), &state.password_hash).unwrap() {
                flash_error_alert("Password was invalid.".to_string(), &mut session);
                return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
            }

            println!(
                "Deleting link {}/{}",
                std::env::var("BASE_URL").unwrap(),
                form.shortname
            );

            let result = ApiClient::new().delete_entry(&form.shortname).await;

            match result {
                Ok(_) => {
                    state.links.remove(&form.shortname);
                    flash_success_alert("Link deleted successfully.".to_string(), &mut session);
                }
                Err(error) => {
                    flash_error_alert(format!("Unexpected error: {}", error), &mut session);
                }
            }

            (StatusCode::FOUND, [(header::LOCATION, "/-/list")]).into_response()
        }
    }
}

// pub(crate) async fn submit_form(
//     mut session: Session,
//     Extension(state): Extension<Arc<Mutex<State>>>,
//     Form(form): Form<FormData>,
// ) -> impl IntoResponse {
//     let mut state = state.lock().unwrap();
//     let _ = ApiClient::new()
//         .delete_entry(&form.shortname)
//         .await
//         .unwrap();

//     // match form.action {
//     //     FormAction::Edit => {
//     //         let link = state.links.get(&form.shortname).unwrap();
//     //         flash_info_alert(
//     //             format!(
//     //                 "Editing link for {}/{}",
//     //                 std::env::var("BASE_URL").unwrap(),
//     //                 form.shortname
//     //             ),
//     //             &mut session,
//     //         );

//     //         flash(
//     //             FlashType::CreateFormUserValues,
//     //             CreateFormRememberValues {
//     //                 shortname: link.shortname.clone(),
//     //                 longurl: link.longurl.clone(),
//     //             },
//     //             &mut session,
//     //         );
//     //         (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
//     //     }

//     //     FormAction::Delete => {
//     //         if !bcrypt::verify(form.password.unwrap_or_default(), &state.password_hash).unwrap() {
//     //             flash_error_alert("Password was invalid.".to_string(), &mut session);
//     //             return (StatusCode::FOUND, [(header::LOCATION, "/-/list")]).into_response();
//     //         }

//     //         // TODO: Send API call to remove link from database
//     //         flash_success_alert("Link deleted successfully.".to_string(), &mut session);
//     //         state.links.remove(&form.shortname);

//     //         (StatusCode::FOUND, [(header::LOCATION, "/-/list")]).into_response()
//     //     }
//     // }
// }
