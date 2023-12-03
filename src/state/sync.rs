use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Extension};
use tokio::sync::Mutex;

use crate::api_client::ApiClient;

use super::{Link, State};

pub(crate) async fn sync_stats(
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let links: Vec<Link> = state.links.values().cloned().collect();
    let response = ApiClient::new().sync_stats(serde_json::json!(links)).await;

    match response {
        Ok(_) => (StatusCode::OK, "success").into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
