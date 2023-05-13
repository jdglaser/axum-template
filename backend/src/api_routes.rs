use std::collections::HashSet;

use axum::{
    routing::{get, post}, Json, Extension, extract::Path, response::IntoResponse
};
use sqlx::Row;

use crate::{models::{Person, CreatePerson}, ApiContext, error::{Result, ResultExt, Error}};

async fn hello_world() -> String {
    "Hello world!".to_string()
}

async fn get_json(Json(payload): Json<Person>) -> Json<Person> {
    Json(payload)
}

async fn get_personby_id(Path(id): Path<u32>, ctx: Extension<ApiContext>) -> Result<Json<Person>> {
    let person: Person = sqlx::query_as(r#"SELECT * FROM person WHERE person_id = $1"#)
        .bind(id)
        .fetch_one(&ctx.db)
        .await?;

    Ok(Json(person))
}

#[axum_macros::debug_handler]
async fn create_person(ctx: Extension<ApiContext>, Json(person): Json<CreatePerson>) -> Result<Json<Person>> {
    tracing::debug!("Creating person");
    let id: u32 = sqlx::query("INSERT INTO person (NAME, AGE, IS_COOL) VALUES ($1, $2, $3) RETURNING person_id")
        .bind(person.name.clone())
        .bind(person.age)
        .bind(person.is_cool)
        .fetch_one(&ctx.db)
        .await
        .on_unique_constraint_error(
            ["person.NAME"],
            |_| Error::unprocessable_entity([("name", format!("Name '{}' is already taken", person.name))])
        )
        .on_unique_constraint_error(
            ["person.NAME", "person.AGE"],
            |_| Error::unprocessable_entity([("name", format!("Name '{}' and age '{}' combo is already taken.", person.name, person.age))])
        )?
        .try_get("PERSON_ID")?;

    tracing::debug!("Found person");
    let person: Person = sqlx::query_as(r#"SELECT * FROM person WHERE person_id = $1"#)
        .bind(id)
        .fetch_one(&ctx.db)
        .await?;

    Ok(Json(person))
}

pub fn api_router() -> axum::Router {
    axum::Router::new()
        .route("/hello", get(hello_world))
        .route("/json-test", post(get_json))
        .route("/person", post(create_person))
}