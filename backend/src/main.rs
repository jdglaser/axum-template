use std::os::macos::raw::stat;

use hyper::{StatusCode, header};
use include_dir::{Dir, include_dir};
use rust_embed::RustEmbed;
use tokio;
use axum::{
    routing::{get}, extract::Path, response::{Response, IntoResponse}, body::{self, Empty, Full}, http::HeaderValue
};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/static"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = axum::Router::new()
        .route("/", get(|| static_path(Path(String::from("index.html")))))
        .route("/api/foo", get(|| ))
        .route("/*path", get(static_path));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    println!("{}", path);
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match Asset::get(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::boxed(Full::from(file.data)))
            .unwrap(),
    }
}