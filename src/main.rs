use askama::Template;
use axum::{
    error_handling::HandleErrorLayer,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    BoxError, Extension, Router,
};

use tower::ServiceBuilder;

use tower_http::services::ServeDir;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, Session, SessionManagerLayer};
use util::html_template::{GlobalTemplateData, HtmlTemplate};

use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

// mod form_submit;
mod api_client;
mod create;
mod list;
mod redirect;
mod state;
mod util;

use state::State;
use util::flash::Alert;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // build our application with a route
    let shared_state = Arc::new(Mutex::new(State::default()));

    let store = MemoryStore::default();

    let session_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(120))),
        );

    state::fetch::fetch_state(&shared_state).await;

    let app = Router::new()
        .route("/", get(create::display_form).post(create::submit_form))
        .nest(
            "/-",
            Router::new()
                .route("/healthcheck", get(handle_healthcheck))
                .route("/state", get(display_state))
                .route("/list", get(list::display_list).post(list::submit_form))
                .route("/about", get(display_about))
                .nest_service("/assets", ServeDir::new("assets"))
                .nest(
                    "/api",
                    Router::new().route("/sync-stats", post(state::sync::sync_stats)),
                )
                .fallback(handle_dashroute_404),
        )
        .route("/*path", get(redirect::redirect))
        .layer(Extension(shared_state))
        .layer(session_layer)
        .fallback(handle_404);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse().unwrap()));

    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "hello!")
}

#[derive(Template)]
#[template(path = "about.jinja")]
struct AboutTemplate {
    global_data: GlobalTemplateData,
}

async fn display_about(session: Session) -> impl IntoResponse {
    let global_data = GlobalTemplateData::fetch(&session);
    HtmlTemplate(AboutTemplate { global_data })
}

async fn display_state(Extension(state): Extension<Arc<Mutex<State>>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&state.lock().await.clone()).unwrap(),
    )
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Route not found.")
}

async fn handle_dashroute_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Route not found.")
}
