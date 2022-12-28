use std::sync::{Arc, Mutex};

use axum::{extract, response::IntoResponse, Extension};
use axum_sessions::extractors::WritableSession;
use hyper::{header, StatusCode};

use crate::{
    state::State,
    util::flash::{flash, Alert, FlashType},
};

pub(crate) async fn redirect(
    mut session: WritableSession,
    extract::OriginalUri(uri): extract::OriginalUri,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    let resolution = golink::resolve(&uri.to_string(), &|value| {
        state
            .lock()
            .unwrap()
            .links
            .get(value)
            .map(|link| link.longurl.clone())
    });

    match resolution {
        Ok(golink::GolinkResolution::RedirectRequest(uri, shortname)) => {
            state
                .lock()
                .unwrap()
                .links
                .entry(shortname)
                .and_modify(|v| v.clicks += 1);

            (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, uri)]).into_response()
        }
        Ok(golink::GolinkResolution::MetadataRequest(shortname)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string_pretty(&state.lock().unwrap().links.get(&shortname).unwrap())
                .unwrap(),
        )
            .into_response(),
        Err(e) => {
            flash(FlashType::Alert, Alert::Error(e.to_string()), &mut session);
            (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, "/")]).into_response()
        }
    }
}
