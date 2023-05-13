use chrono::Utc;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

use crate::serde_formats::{datetime_format, option_datetime_format};

#[derive(Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "UPPERCASE")]
pub struct Person {
    #[sqlx(rename="person_id")]
    pub id: u32,
    pub name: String,
    pub age: u8,
    pub is_cool: bool,
    #[serde(with="datetime_format")]
    pub created_at: chrono::DateTime<Utc>,
    #[serde(with="option_datetime_format")]
    pub modified_at: Option<chrono::DateTime<Utc>>
}

#[derive(Serialize, Deserialize)]
pub struct CreatePerson {
    pub name: String,
    pub age: u8,
    pub is_cool: bool,
}