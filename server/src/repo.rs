use sqlx::{
	types::{
		chrono::{DateTime, Utc},
		Uuid,
	},
	PgPool,
};

use crate::models::Benchmark;

#[derive(Clone)]
pub struct BenchmarkRepo {
	pool: PgPool,
}

impl BenchmarkRepo {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	pub async fn get_all(&self) -> anyhow::Result<Vec<Benchmark>> {
		Ok(sqlx::query_as!(
			Benchmark,
			"
SELECT * FROM benchmarks
			"
		)
		.fetch_all(&self.pool)
		.await?)
	}

	pub async fn get_by_year(&self, year: i64) -> anyhow::Result<Vec<Benchmark>> {
		Ok(sqlx::query_as!(
			Benchmark,
			"
SELECT * FROM benchmarks 
WHERE	year = $1
			",
			year,
		)
		.fetch_all(&self.pool)
		.await?)
	}

	pub async fn get_by_triplet(
		&self,
		username: &str,
		year: i64,
		day: i64,
	) -> anyhow::Result<Benchmark> {
		Ok(sqlx::query_as!(
			Benchmark,
			"
SELECT * FROM benchmarks 
WHERE	username = $1 AND year = $2 AND day = $3
			",
			username,
			year,
			day
		)
		.fetch_one(&self.pool)
		.await?)
	}

	pub async fn create<'a>(&self, data: &BenchmarkData<'a>) -> anyhow::Result<Benchmark> {
		Ok(sqlx::query_as!(
			Benchmark,
			"
INSERT INTO benchmarks (
  username, year, day, p1, p1_deviation, p2, p2_deviation, parse, parse_deviation, last_run, timeout
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
RETURNING *
			",
			data.username,
			data.year,
			data.day,
			data.p1,
			data.p1_deviation,
			data.p2,
			data.p2_deviation,
			data.parse,
			data.parse_deviation,
			data.last_run,
			data.timeout
		)
		.fetch_one(&self.pool)
		.await?)
	}

	pub async fn update<'a>(
		&self,
		id: &Uuid,
		data: &BenchmarkData<'a>,
	) -> anyhow::Result<Benchmark> {
		Ok(sqlx::query_as!(
			Benchmark,
			"
UPDATE benchmarks
SET
	username = $2,
	year = $3,
	day = $4,
	p1 = $5,
	p1_deviation = $6,
	p2 = $7,
	p2_deviation = $8,
	parse = $9,
	parse_deviation = $10,
	last_run = $11,
	timeout = $12
WHERE id = $1
RETURNING *
			",
			id,
			data.username,
			data.year,
			data.day,
			data.p1,
			data.p1_deviation,
			data.p2,
			data.p2_deviation,
			data.parse,
			data.parse_deviation,
			data.last_run,
			data.timeout
		)
		.fetch_one(&self.pool)
		.await?)
	}
}

pub struct BenchmarkData<'a> {
	pub username: &'a str,
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
}
