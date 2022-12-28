use std::sync::{Arc, Mutex};

use crate::{
    state::{Link, State},
    util::html_template::HtmlTemplate,
};
use askama::Template;
use axum::{response::IntoResponse, Extension};
use axum_sessions::extractors::WritableSession;

use super::submit_form::{CreateFormRememberValues, CreateFormUserFeedback};

#[derive(Template)]
#[template(path = "create_link.jinja")]
struct CreateTemplate {
    message: Option<CreateFormUserFeedback>,
    form_user_values: CreateFormRememberValues,
    popular: Vec<Link>,
    recent: Vec<Link>,
    base_url: String,
}

pub(crate) async fn display_form(
    mut session: WritableSession,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let message: Option<CreateFormUserFeedback> = session.get("form_submit");
    let form_user_values: CreateFormRememberValues =
        session.get("form_user_values").unwrap_or_default();

    let state = state.lock().unwrap();

    let mut recent: Vec<Link> = state.links.values().cloned().collect();
    recent.sort_by_key(|link| link.created_at.clone());
    recent.reverse();
    recent.truncate(5);

    let mut popular: Vec<Link> = state.links.values().cloned().collect();
    popular.sort_by_key(|link| link.clicks);
    popular.reverse();
    popular.truncate(5);

    session.remove("form_submit");

    HtmlTemplate(CreateTemplate {
        message,
        form_user_values,
        popular,
        recent,
        base_url: std::env::var("BASE_URL").unwrap(),
    })
}
