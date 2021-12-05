use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::time::Duration;

use bencher::{run_benchmark, BenchError};
use chrono::Utc;
use serde::Deserialize;
use shared::BenchStats;
use sqlx::PgPool;
use tokio::fs::read_to_string;
use tokio::time::sleep;

use crate::repo::{BenchmarkData, BenchmarkRepo};

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
enum Lang {
	Rust,
}

impl Display for Lang {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Lang::Rust => write!(f, "rust"),
		}
	}
}

type Users<'a> = HashMap<&'a str, HashMap<i64, HashMap<Lang, &'a str>>>;

const BENCH_CACHE_HOURS: i64 = 24;

pub async fn poll_benchmark(pool: PgPool) -> ! {
	let benchmark_repo = BenchmarkRepo::new(pool);
	let mut first = true;
	loop {
		if first {
			first = false;
		} else {
			sleep(Duration::from_secs(600)).await;
		}

		let config_file = match read_to_string("config.ron").await {
			Ok(config_file) => config_file,
			Err(_) => {
				println!("Could not read config.ron");
				continue;
			}
		};

		let users: Users = match ron::from_str(&config_file) {
			Ok(users) => users,
			Err(e) => {
				println!("Invalid config.ron: {}", e);
				continue;
			}
		};

		let aoc_session = match env::var("AOC_SESSION") {
			Ok(aoc_session) => aoc_session,
			Err(_) => {
				println!("AOC_SESSION variable missing");
				continue;
			}
		};

		for (user, years) in users {
			for (year, langs) in years {
				for (lang, docker_image) in langs {
					for day in 1..=25 {
						let existing_result =
							benchmark_repo.get_by_triplet(user, year, day).await.ok();
						let now = Utc::now();
						if let Some(existing_result) = &existing_result {
							let hours_since_last_bench =
								(now - existing_result.last_run).num_hours();
							let needs_update = if existing_result.timeout {
								hours_since_last_bench <= 5 * BENCH_CACHE_HOURS
							} else {
								hours_since_last_bench <= BENCH_CACHE_HOURS
							};

							if needs_update {
								continue;
							}
						}

						match run_benchmark(docker_image, day, &aoc_session).await {
							Ok(bench) => {
								let transform_bench =
									|bench: Option<BenchStats>| -> (Option<i64>, Option<i64>) {
										bench
											.map(|p1| (Some(p1.average_ns), Some(p1.deviation_ns)))
											.unwrap_or_default()
									};
								let (p1, p1_deviation) = transform_bench(bench.p1);
								let (p2, p2_deviation) = transform_bench(bench.p2);
								let (parse, parse_deviation) = transform_bench(bench.parse);
								let data = BenchmarkData {
									username: user,
									year,
									day,
									p1,
									p1_deviation,
									p2,
									p2_deviation,
									parse,
									parse_deviation,
									last_run: now,
									timeout: false,
								};
								if let Some(existing_result) = existing_result {
									let _ = benchmark_repo.update(&existing_result.id, &data).await;
								} else {
									let _ = benchmark_repo.create(&data).await;
								}
							}
							Err(BenchError::Timeout) => {
								let data = BenchmarkData {
									username: user,
									year,
									day,
									p1: None,
									p1_deviation: None,
									p2: None,
									p2_deviation: None,
									parse: None,
									parse_deviation: None,
									last_run: now,
									timeout: true,
								};
								let _ = benchmark_repo.create(&data).await;
							}
							Err(_) => println!(
								"Benchmark failed: image: {}, year: {}, day: {}, lang: {}",
								docker_image, year, day, lang
							),
						}
					}
				}
			}
		}
	}
}
