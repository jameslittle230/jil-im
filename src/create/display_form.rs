use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    state::{Link, State},
    util::{
        flash::{clear_flash, Alert, FlashType},
        html_template::{GlobalTemplateData, HtmlTemplate},
    },
};
use askama::Template;
use axum::{response::IntoResponse, Extension};
use tower_sessions::Session;

use super::FormValues;

#[derive(Template)]
#[template(path = "create_link.jinja")]
struct CreateTemplate {
    global_data: GlobalTemplateData,
    form_values: FormValues,
    popular: Vec<Link>,
    recent: Vec<Link>,
}

pub(crate) async fn display_form(
    mut session: Session,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let global_data = GlobalTemplateData::fetch(&session);
    let form_values: FormValues = session
        .get(&(FlashType::CreateFormUserValues.to_string()))
        .expect("infallible")
        .unwrap_or_default();

    let state = state.lock().await;

    let mut recent: Vec<Link> = state.links.values().cloned().collect();
    recent.sort_by_key(|link| link.created_at);
    recent.reverse();
    recent.truncate(5);

    let mut popular: Vec<Link> = state.links.values().cloned().collect();
    popular.sort_by_key(|link| usize::MAX - link.clicks);
    popular.truncate(5);

    clear_flash(&mut session);

    HtmlTemplate(CreateTemplate {
        global_data,
        form_values,
        popular,
        recent,
    })
}
