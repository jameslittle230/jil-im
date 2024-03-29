use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    state::{Link, State},
    util::{
        flash::{clear_flash, Alert},
        html_template::{GlobalTemplateData, HtmlTemplate},
    },
};
use askama::Template;
use axum::{response::IntoResponse, Extension};
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "display_list.jinja")]
struct CreateTemplate {
    global_data: GlobalTemplateData,
    links: Vec<Link>,
}

pub(crate) async fn display_list(
    mut session: Session,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let global_data = GlobalTemplateData::fetch(&session);
    let state = state.lock().await;

    let mut links: Vec<Link> = state.links.values().cloned().collect();
    links.sort_by_key(|link| link.created_at);
    links.reverse();

    clear_flash(&mut session);

    HtmlTemplate(CreateTemplate { global_data, links })
}
