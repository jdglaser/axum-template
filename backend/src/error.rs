use std::{collections::{HashMap, HashSet}, borrow::Cow};

use anyhow::{anyhow, bail};
use axum::{response::{IntoResponse, Response}, body::Full, Json};
use hyper::{StatusCode, body::Bytes, Body, header::Iter};
use regex::Regex;
use sqlx::error::DatabaseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl Error {
    /// Helper function for creating an Error::UnprocessableEntity from a collection
    /// of tuples
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    /// Convert Error variant to actual Axum StatusCode
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Tell Axum how to turn our custom errors into a response
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::UnprocessableEntity { errors } => {
                #[derive(serde::Serialize)]
                struct Errors {
                    errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
                }

                return (StatusCode::UNPROCESSABLE_ENTITY, Json(Errors { errors })).into_response();
            }

            Self::Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
            }

            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub trait ResultExt<'a, T> {
    fn on_unique_constraint_error(
        self, 
        constraint_names: impl IntoIterator<Item = &'a str>, 
        f: impl FnOnce(&Box<dyn DatabaseError>) -> Error
    ) -> Result<T, Error>;
}

fn convert_constraint_error<'a>(message: &'a str) -> anyhow::Result<HashSet<&'a str>> {
    let re = Regex::new(r"UNIQUE constraint failed:.+?(?P<constraints>.+)").unwrap();
    Ok(re.captures(message)
        .ok_or(anyhow!("foo"))?
        .name("constraints")
        .ok_or(anyhow!("f"))?
        .as_str()
        .split(", ")
        .collect())
}

impl<'a, T, E> ResultExt<'a, T> for Result<T, E>
where
    E: Into<Error>  
{
    /// Convert a database constraint error to a custom Error defined by `f`.
    /// If the error is not a constraint error, or does not contain the constraints
    /// specified by `constraint_names`, then the error is passed through as normal.
    fn on_unique_constraint_error(
        self,
        constraint_names: impl IntoIterator<Item = &'a str>, 
        f: impl FnOnce(&Box<dyn DatabaseError>) -> Error
    ) -> Result<T, Error> {
        self.map_err(|e| {
            match e.into() {
                Error::Sqlx(se) => {
                    if let sqlx::Error::Database(ref dbe) = se {
                        let message = dbe.message();
                        let res = convert_constraint_error(message);
    
                        if res.is_ok() && res.unwrap() == constraint_names.into_iter().collect() {
                            return f(dbe);
                        }
                    }
                    se.into()
                },
                e => e,
            }
        })
    }
}