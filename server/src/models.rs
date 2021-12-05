use serde::Serialize;
use sqlx::types::{
	chrono::{DateTime, Utc},
	Uuid,
};

#[derive(Debug, Serialize)]
pub struct Benchmark {
	pub id: Uuid,
	pub username: String,
	pub year: i64,
	pub day: i64,
	pub p1: Option<i64>,
	pub p1_deviation: Option<i64>,
	pub p2: Option<i64>,
	pub p2_deviation: Option<i64>,
	pub parse: Option<i64>,
	pub parse_deviation: Option<i64>,
	pub last_run: DateTime<Utc>,
	pub timeout: bool,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}
