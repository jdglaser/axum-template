use std::fs;

use axum::{
    routing::get, extract::Path, Extension
};

mod api_routes;
mod serde_formats;
mod static_route;
mod models;
mod error;

use api_routes::api_router;
use error::Error;
use static_route::static_router;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Clone)]
struct ApiContext {
    db: SqlitePool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug".into()),
        )
        .with(fmt::layer())
        .init();

    fs::write("data.db", "")?;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://data.db?mode=rwc").await?;

    sqlx::migrate!()
        .run(&pool)
        .await?;

    let app = axum::Router::new()
        .nest("/api", api_router())
        .route("/", get(|| static_router(Path(String::from("index.html")))))
        .route("/*path", get(static_router))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(ApiContext {db: pool}))
                .layer(TraceLayer::new_for_http())
        );

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use regex::Regex;

    #[test]
    fn parse_e() {
        let test_str = r#"UNIQUE constraint failed: person.NAME, person.AGE"#;
        let re = Regex::new(r"UNIQUE constraint failed:.+?(?P<constraints>.+)").unwrap();
        println!("{:?}", re.is_match(test_str));

        println!("{:?}", re.captures(test_str).unwrap().name("constraints").unwrap().as_str().split(", ").collect::<HashSet<&str>>());
    }
}