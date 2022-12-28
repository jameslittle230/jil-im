use std::sync::{Arc, Mutex};

use axum::{extract::Form, response::IntoResponse, Extension};
use axum_sessions::extractors::WritableSession;
use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::{
    state::Link,
    util::flash::{flash, FlashType},
    State,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct CreateForm {
    shortname: String,
    longurl: String,
    password: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct CreateFormRememberValues {
    pub(crate) shortname: String,
    pub(crate) longurl: String,
}

impl From<&CreateForm> for CreateFormRememberValues {
    fn from(value: &CreateForm) -> Self {
        Self {
            shortname: value.shortname.clone(),
            longurl: value.longurl.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) enum CreateFormUserFeedback {
    Success(String),
    Error(String),
}

#[derive(Serialize, Deserialize)]
struct CreateEntrySuccessApiResponse {
    longurl: String,
    shortname: String,
    created_at: String, // TODO: make this a chrono type
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
    Success(CreateEntrySuccessApiResponse),
    Error(ApiError),
}

pub(crate) async fn submit_form(
    mut session: WritableSession,
    Extension(state): Extension<Arc<Mutex<State>>>,
    Form(mut form): Form<CreateForm>,
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
        CreateFormRememberValues::from(&form),
        &mut session,
    );

    if bcrypt::verify(&form.password, &state.lock().unwrap().password_hash).unwrap() == false {
        flash(
            FlashType::CreateFormUserFeedback,
            CreateFormUserFeedback::Error("Password was invalid.".to_string()),
            &mut session,
        );

        return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
    }

    let client = reqwest::Client::new();
    let result = client
        .post(format!(
            "{}/shortener/entries",
            std::env::var("JIL_API_URL").unwrap()
        ))
        .json(&form)
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("JIL_API_ADMIN_BEARER_TOKEN").unwrap()
            ),
        )
        .send()
        .await;

    match result {
        Ok(response) => {
            let result = response.json::<CreateEntryApiResponse>().await;

            match result {
                Ok(api_response) => match api_response {
                    CreateEntryApiResponse::Success(success_response) => {
                        flash(
                            FlashType::CreateFormUserFeedback,
                            CreateFormUserFeedback::Success(format!(
                                "{}/{}",
                                std::env::var("BASE_URL").unwrap(),
                                form.shortname
                            )),
                            &mut session,
                        );

                        let mut state = state.lock().unwrap();

                        state.links.insert(
                            success_response.shortname.clone(),
                            Link {
                                shortname: success_response.shortname.clone(),
                                longurl: success_response.longurl.clone(),
                                created_at: success_response.created_at.clone(),
                                clicks: 0,
                            },
                        );
                    }
                    CreateEntryApiResponse::Error(api_error) => {
                        flash(
                            FlashType::CreateFormUserFeedback,
                            CreateFormUserFeedback::Error(api_error.message),
                            &mut session,
                        );
                    }
                },
                Err(error) => {
                    flash(
                        FlashType::CreateFormUserFeedback,
                        CreateFormUserFeedback::Error(format!("Unexpected error: {:?}", error)),
                        &mut session,
                    );
                }
            }
        }
        Err(error) => {
            flash(
                FlashType::CreateFormUserFeedback,
                CreateFormUserFeedback::Error(format!("Unexpected error: {:?}", error)),
                &mut session,
            );
        }
    };

    (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
}
