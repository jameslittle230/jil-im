use std::sync::Arc;
use tokio::sync::Mutex;

use axum::http::{header, StatusCode};
use axum::{extract::Form, response::IntoResponse, Extension};

use serde::{Deserialize, Serialize};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tower_sessions::Session;

use crate::api_client::ApiClient;
use crate::util::flash::{flash_error_alert, flash_success_alert, Alert};
use crate::{
    state::Link,
    util::flash::{flash, FlashType},
    State,
};

use super::FormValues;

#[derive(Serialize, Deserialize)]
pub(crate) struct FormData {
    shortname: String,
    longurl: String,
    password: String,
}

impl From<&FormData> for FormValues {
    fn from(value: &FormData) -> Self {
        Self {
            shortname: value.shortname.clone(),
            longurl: value.longurl.clone(),
            shortname_is_disabled: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ApiError {
    error: bool,
    code: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CreateEntryApiResponse {
    Success(Link),
    Error(ApiError),
}

pub(crate) async fn submit_form(
    mut session: Session,
    Extension(state): Extension<Arc<Mutex<State>>>,
    Form(mut form): Form<FormData>,
) -> impl IntoResponse {
    if form.shortname.is_empty() {
        form.shortname = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .map(|c| c.to_ascii_lowercase())
            .collect();
    }

    flash(
        FlashType::CreateFormUserValues,
        FormValues::from(&form),
        &mut session,
    );

    if !bcrypt::verify(&form.password, &state.lock().await.password_hash).unwrap() {
        flash_error_alert("Password was invalid.".to_string(), &mut session);
        return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
    }

    let result = ApiClient::new()
        .create_entry(&form.shortname, &form.longurl)
        .await;

    match result {
        Ok(link) => {
            flash_success_alert(
                format!("{}/{}", std::env::var("BASE_URL").unwrap(), form.shortname),
                &mut session,
            );

            let mut state = state.lock().await;
            state.links.insert(link.shortname.clone(), link);
        }

        Err(error) => {
            flash(
                FlashType::Alert,
                Alert::Error(format!("Unexpected error: {:?}", error)),
                &mut session,
            );
        }
    }

    (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
}
