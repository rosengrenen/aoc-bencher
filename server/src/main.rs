mod http;
mod models;
mod repo;
mod runner;

use std::env;

use runner::poll_benchmark;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  env_logger::init();
	let _ = dotenv::dotenv();
	let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is missing");
	let db_pool = PgPool::connect(&db_url).await?;

	tokio::spawn({
		let db_pool = db_pool.clone();
		async move { poll_benchmark(db_pool).await }
	});

	let routes = http::init_routes(db_pool);
	warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

	Ok(())
}
