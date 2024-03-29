use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use tower_sessions::Session;

use super::flash::{Alert, FlashType};

pub(crate) struct HtmlTemplate<T>(pub(crate) T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

pub(crate) struct GlobalTemplateData {
    pub(crate) alert: Option<Alert>,
    pub(crate) base_url: String,
}

impl GlobalTemplateData {
    pub(crate) fn fetch(session: &Session) -> Self {
        Self {
            alert: session
                .get(FlashType::Alert.to_string().as_str())
                .expect("infallible"),
            base_url: std::env::var("BASE_URL").unwrap(),
        }
    }
}
