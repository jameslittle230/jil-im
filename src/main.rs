use axum::{response::Html, routing::get, Extension, Router};
use std::{
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
        .route("/", get(handler))
        .layer(Extension(shared_state));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(Extension(state): Extension<Arc<Mutex<State>>>) -> Html<String> {
    let mut state = state.lock().unwrap();
    state.view_count += 1;
    let view_count = state.view_count;
    Html(format!(
        "<h1>Hello, World! You are viewer number {view_count}.</h1>"
    ))
}
