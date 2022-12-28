use axum::{
    response::IntoResponse,
    routing::{get, get_service, post},
    Extension, Router,
};

use axum_sessions::{async_session::MemoryStore, SessionLayer};

use tower_http::services::ServeDir;

use hyper::{header, StatusCode};

use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

// mod form_submit;
mod create;
mod list;
mod redirect;
mod state;
mod util;

use state::State;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // build our application with a route
    let shared_state = Arc::new(Mutex::new(State::default()));

    let store = MemoryStore::new();
    let secret = b"
93 ad b2 56 79 30 85 5d 02 d1 0f e5 52 80 75 5b 
80 bf 93 f7 b0 62 9d 42 fd e6 eb 10 6c 2f 6c 3d 
da 6e 12 ed 0d d3 e7 93 eb 8e 97 bb 32 db 7f ca 
d9 14 7d 26 2b 61 3b c4 eb 51 ae eb b9 ac ac 15"; // MUST be at least 64 bytes!
    let session_layer = SessionLayer::new(store, secret);

    state::fetch::fetch_state(&shared_state).await;

    let app = Router::new()
        .route("/", get(create::display_form).post(create::submit_form))
        .nest(
            "/-",
            Router::new()
                .route("/healthcheck", get(handle_healthcheck))
                .route("/state", get(display_state))
                .route("/list", get(list::display_list).post(list::submit_form))
                // .route("/about", get(display_about))
                .nest_service(
                    "/assets",
                    get_service(ServeDir::new("assets")).handle_error(handle_error),
                )
                .nest(
                    "/api",
                    Router::new()
                        .route("/redirects", get(list_redirects))
                        .route("/redirects", post(create_redirects))
                        .route("/redirects/:key", get(retrieve_redirect))
                        .route("/redirects/:key/update", post(update_redirect))
                        .route("/redirects/:key/delete", post(delete_redirect)),
                )
                .fallback(handle_dashroute_404),
        )
        .route("/*path", get(redirect::redirect))
        .layer(Extension(shared_state))
        .layer(session_layer)
        .fallback(handle_404);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn handle_healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "hello!")
}

async fn display_state(Extension(state): Extension<Arc<Mutex<State>>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&state.lock().unwrap().to_owned()).unwrap(),
    )
}

async fn create_redirects() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "not implemented")
}
async fn list_redirects() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "not implemented")
}
async fn retrieve_redirect() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "not implemented")
}
async fn update_redirect() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "not implemented")
}
async fn delete_redirect() -> impl IntoResponse {
    StatusCode::TEMPORARY_REDIRECT
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Route not found.")
}

async fn handle_dashroute_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Route not found.")
}
