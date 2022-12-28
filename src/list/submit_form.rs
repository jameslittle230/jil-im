use std::sync::{Arc, Mutex};

use axum::{response::IntoResponse, Extension, Form};
use axum_sessions::extractors::WritableSession;
use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    state::State,
    util::flash::{flash, Alert, FlashType},
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

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct CreateFormRememberValues {
    pub(crate) shortname: String,
    pub(crate) longurl: String,
}

pub(crate) async fn submit_form(
    mut session: WritableSession,
    Extension(state): Extension<Arc<Mutex<State>>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();

    match form.action {
        FormAction::Edit => {
            let link = state.links.get(&form.shortname).unwrap();
            flash(
                FlashType::Alert,
                Alert::Info(format!(
                    "Editing link for {}/{}",
                    std::env::var("BASE_URL").unwrap(),
                    form.shortname
                )),
                &mut session,
            );

            flash(
                FlashType::CreateFormUserValues,
                CreateFormRememberValues {
                    shortname: link.shortname.clone(),
                    longurl: link.longurl.clone(),
                },
                &mut session,
            );
            return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
        }

        FormAction::Delete => {
            if bcrypt::verify(&form.password.unwrap_or_default(), &state.password_hash).unwrap()
                == false
            {
                flash(
                    FlashType::Alert,
                    Alert::Error("Password was invalid.".to_string()),
                    &mut session,
                );
            } else {
                // TODO: Send API call to remove link from database
                flash(
                    FlashType::Alert,
                    Alert::Success("Link deleted successfully.".to_string()),
                    &mut session,
                );
                state.links.remove(&form.shortname);
            }
            return (StatusCode::FOUND, [(header::LOCATION, "/-/list")]).into_response();
        }
    }
}
