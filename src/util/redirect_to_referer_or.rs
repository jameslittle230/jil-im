use axum::{http::HeaderValue, response::IntoResponse};
use hyper::{header, StatusCode};

pub(crate) fn redirect_to_referer_or(
    route: &str,
    referer: Option<&HeaderValue>,
) -> impl IntoResponse {
    if let Some(referer) = referer {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, referer)],
        )
            .into_response();
    }

    (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, route)]).into_response()
}
