use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Use the first arg as the directory to serve files from
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());

    // Get the port to listen on from the environment, default 8000
    let port = std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(8000);

    let serve_dir = get_service(ServeDir::new(dir)).handle_error(handle_error);
    let app = Router::new()
        .route("/health", get(health))
        .fallback_service(serve_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "OK"
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}
