use sqlx::PgPool;
use warp::Filter;

pub fn init_routes(
	pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	filters::benchmarks_all(pool.clone()).or(filters::benchmarks_year(pool.clone()))
}

mod filters {
	use std::convert::Infallible;

	use sqlx::PgPool;
	use warp::Filter;

	use super::handlers;

	pub fn benchmarks_all(
		pool: PgPool,
	) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
		warp::path!("benchmarks")
			.and(warp::get())
			.and(with_pool(pool))
			.and_then(handlers::all_benchmarks)
	}

	pub fn benchmarks_year(
		pool: PgPool,
	) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
		warp::path!("benchmarks" / i64)
			.and(warp::get())
			.and(with_pool(pool))
			.and_then(handlers::year_benchmarks)
	}

	fn with_pool(pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
		warp::any().map(move || pool.clone())
	}
}

mod handlers {
	use std::convert::Infallible;

	use sqlx::PgPool;

	use crate::{models::Benchmark, repo::BenchmarkRepo};

	pub async fn all_benchmarks(pool: PgPool) -> Result<impl warp::Reply, Infallible> {
		let benchmarks_repo = BenchmarkRepo::new(pool);
		let benchmarks = benchmarks_repo.get_all().await.unwrap_or_else(|_| vec![]);
		let benchmarks = filter_benchmarks(benchmarks);
		Ok(warp::reply::json(&benchmarks))
	}

	pub async fn year_benchmarks(year: i64, pool: PgPool) -> Result<impl warp::Reply, Infallible> {
		let benchmarks_repo = BenchmarkRepo::new(pool);
		let benchmarks = benchmarks_repo
			.get_by_year(year)
			.await
			.unwrap_or_else(|_| vec![]);
		let benchmarks = filter_benchmarks(benchmarks);
		Ok(warp::reply::json(&benchmarks))
	}

	fn filter_benchmarks(benchmarks: Vec<Benchmark>) -> Vec<Benchmark> {
		benchmarks
			.into_iter()
			.filter(|benchmark| benchmark.p1.is_some() || benchmark.p2.is_some())
			.collect()
	}
}
