use std::sync::{Arc, Mutex};

use axum::{extract::Form, response::IntoResponse, Extension};
use axum_sessions::extractors::WritableSession;
use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::{state::Link, State};

#[derive(Serialize, Deserialize)]
pub(crate) struct CreateForm {
    shortname: String,
    longurl: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum CreateFormFlashResponse {
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

    if bcrypt::verify(&form.password, &state.lock().unwrap().password_hash).unwrap() == false {
        let _ = session.insert(
            "form_submit",
            CreateFormFlashResponse::Error("Password was invalid.".to_string()),
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
                        let _ = session.insert(
                            "form_submit",
                            CreateFormFlashResponse::Success(format!(
                                "{}/{}",
                                std::env::var("BASE_URL").unwrap(),
                                form.shortname
                            )),
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
                        let _ = session.insert(
                            "form_submit",
                            CreateFormFlashResponse::Error(api_error.message),
                        );
                    }
                },
                Err(error) => {
                    let _ = session.insert(
                        "form_submit",
                        CreateFormFlashResponse::Error(format!("Unexpected error: {:?}", error)),
                    );
                }
            }
        }
        Err(error) => {
            let _ = session.insert(
                "form_submit",
                CreateFormFlashResponse::Error(format!("Unexpected error: {:?}", error)),
            );
        }
    };

    (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response()
}
