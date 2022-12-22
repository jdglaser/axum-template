use std::os::macos::raw::stat;

use hyper::{StatusCode, header};
use include_dir::{Dir, include_dir};
use rust_embed::RustEmbed;
use tokio;
use axum::{
    routing::{get}, extract::Path, response::{Response, IntoResponse}, body::{self, Empty, Full}, http::HeaderValue
};

mod static_route;

use static_route::static_route;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/static"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_router = axum::Router::new()
        .route("/foo", get(|| async {"Hello world!"}));

    let app = axum::Router::new()
        .nest("/api", api_router)
        .route("/", get(|| static_route(Path(String::from("index.html")))))
        .route("/*path", get(static_route));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}