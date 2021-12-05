use std::collections::HashMap;

use serde::Deserialize;
use tokio::fs::read_to_string;

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
enum Lang {
	Rust,
}

type Users<'a> = HashMap<&'a str, HashMap<i64, HashMap<Lang, &'a str>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config_file = read_to_string("config.ron").await?;
	let users: Users = ron::from_str(&config_file)?;
	let aoc_session = std::env::var("AOC_SESSION").unwrap();
	let year = 2021;
	let day = 2;
	for (user, years) in users {
		let langs = match years.get(&year) {
			Some(langs) => langs,
			None => continue,
		};

		for (lang, docker_image) in langs {
			match bencher::run_benchmark(docker_image, day, &aoc_session).await {
				Ok(bench) => println!("{} {:?} {:?}", user, lang, bench),
				Err(_) => println!("benchmark failed"),
			};
		}
	}

	Ok(())
}
