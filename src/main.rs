use askama::Template;

use axum::{
    extract,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service, post},
    Extension, Router,
};

use tower_http::services::ServeDir;

use hyper::{header, StatusCode};

use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Default)]
struct State {
    view_count: usize,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let shared_state = Arc::new(Mutex::new(State::default()));
    let app = Router::new()
        .route("/", get(display_create_form))
        .route("/-/admin/view", get(display_list))
        .nest_service(
            "/-/assets",
            get_service(ServeDir::new("assets")).handle_error(handle_error),
        )
        .nest(
            "/-/api",
            Router::new()
                .route("/redirects", get(list_redirects))
                .route("/redirects", post(create_redirects))
                .route("/redirects/:key", get(retrieve_redirect))
                .route("/redirects/:key/update", post(update_redirect))
                .route("/redirects/:key/delete", post(delete_redirect)),
        )
        .route("/:key", get(redirect))
        .layer(Extension(shared_state))
        .fallback(handle_api_404);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn display_create_form() -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "create.html")]
    struct CreateTemplate {}

    HtmlTemplate(CreateTemplate {})
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn display_list() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "not implemented")
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

async fn redirect(
    extract::OriginalUri(uri): extract::OriginalUri,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> Response {
    let mut state = state.lock().unwrap();
    state.view_count += 1;

    fn lookup(input: &str) -> Option<String> {
        match input {
            "asdf" => Some("https://google.com".to_string()),
            "foo" => Some("https://jameslittle.me".to_string()),
            _ => None,
        }
    }

    let url = format!("http://go{uri}");
    dbg!(&url);

    let resolution = golink::resolve(&url, &lookup);

    dbg!(&resolution);

    match resolution {
        Ok(golink::GolinkResolution::RedirectRequest(uri)) => {
            (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, uri)]).into_response()
        }
        _ => (StatusCode::NOT_IMPLEMENTED, "not implemented").into_response(),
    }
}

async fn handle_api_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Route not found.")
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

struct HtmlTemplate<T>(T);

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
