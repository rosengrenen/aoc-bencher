use std::{
	collections::HashMap,
	fs::{create_dir_all, read_dir, read_to_string, File},
	io::Write,
	path::{Path, PathBuf},
	process::Command,
	time::UNIX_EPOCH,
};

use rouille::router;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct RepoInfo {
	path: PathBuf,
	bench_command: String,
}

#[derive(Debug, Deserialize)]
struct Config {
	repos: HashMap<String, RepoInfo>,
}

fn main() {
	std::thread::spawn(run_benchmarks_interval);
	rouille::start_server("localhost:8080", move |request| {
		router!(
			request,
			(GET) (/api/days/all) => {
				let mut days = HashMap::new();
				for day in 0..=25 {
					match read_day_benchmarks(day) {
						Ok(benchmarks) => {
							days.insert(day, benchmarks);
						}
						Err(_) => continue,
					}
				}
				rouille::Response::json(&days)
			},
			(GET) (/api/days/{day: i64}) => {
				match read_day_benchmarks(day) {
					Ok(benchmarks) => rouille::Response::json(&benchmarks),
					Err(_) => rouille::Response::empty_400(),
				}
			},
			_ => rouille::Response::empty_404()
		)
	});
}

fn read_day_benchmarks(day: i64) -> anyhow::Result<Vec<SavedBench>> {
	let entries = read_dir(format!("output/day{:02}", day))?;
	let mut benches = vec![];
	for entry in entries {
		let entry = match entry {
			Ok(entry) => entry,
			Err(_) => continue,
		};

		if !entry.path().is_file() {
			continue;
		}

		let filename = entry.file_name();
		let filename = filename.to_string_lossy().to_string();

		if !filename.ends_with(".json") {
			continue;
		}

		let content = read_to_string(format!("output/day{:02}/{}", day, filename))?;
		let bench: SavedBench = serde_json::from_str(&content)?;
		benches.push(bench);
	}

	Ok(benches)
}

fn run_benchmarks_interval() {
	loop {
		std::thread::sleep(std::time::Duration::from_secs(600));
		for day in 1..=25 {
			let _ = run_benchmarks(day);
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct SavedBench {
	when: i64,
	day: i64,
	name: String,
	benchmark: Bench,
}

fn run_benchmarks(day: i64) -> anyhow::Result<()> {
	let config = read_to_string("config.json")?;
	let config: Config = serde_json::from_str(&config)?;

	let one_day_seconds = 60 * 60 * 24;
	let now = timestamp_now();
	let old_benches = read_day_benchmarks(day).unwrap_or_else(|_| vec![]);
	let skip_benches: Vec<_> = old_benches
		.iter()
		.filter(|bench| bench.when + one_day_seconds > now)
		.collect();

	for (name, repo) in config.repos {
		if skip_benches
			.iter()
			.find(|bench| bench.name == name)
			.is_some()
		{
			continue;
		}

		if let Err(e) = git_pull(&repo.path) {
			eprintln!("{}", e);
			continue;
		}

		let benchmark = match run_benchmark(&repo.path, &repo.bench_command, day) {
			Ok(benchmark) => benchmark,
			Err(e) => {
				eprintln!("{}", e);
				continue;
			}
		};

		if benchmark.p1.is_none() && benchmark.p2.is_none() && benchmark.parse.is_none() {
			continue;
		}

		create_dir_all(format!("output/day{:02}", day))?;

		let now = timestamp_now();
		let mut file = File::create(format!("output/day{:02}/{}.json", day, name))?;
		file.write_all(
			serde_json::to_string(&SavedBench {
				when: now,
				day,
				name,
				benchmark,
			})?
			.as_bytes(),
		)?;
	}

	Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct BenchStats {
	average_ns: i64,
	deviation_ns: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Bench {
	p1: Option<BenchStats>,
	p2: Option<BenchStats>,
	parse: Option<BenchStats>,
}

fn git_pull(repo_path: &Path) -> anyhow::Result<()> {
	let output = Command::new("sh")
		.arg("-c")
		.arg("git pull")
		.current_dir(repo_path)
		.output()?;
	if !output.status.success() {
		anyhow::bail!("Failed to git pull");
	}

	Ok(())
}

fn run_benchmark(repo_path: &Path, command: &str, day: i64) -> anyhow::Result<Bench> {
	let mut command_parts: Vec<String> = command
		.split_whitespace()
		.filter(|part| !part.is_empty())
		.map(|part| part.to_string())
		.collect();
	command_parts.push(day.to_string());
	let output = Command::new("sh")
		.arg("-c")
		.arg(command_parts.join(" "))
		.current_dir(repo_path)
		.output()?;
	if !output.status.success() {
		anyhow::bail!("Failed to benchmark");
	}

	let output = std::str::from_utf8(&output.stdout)?;
	Ok(serde_json::from_str(&output)?)
}

fn timestamp_now() -> i64 {
	std::time::SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64
}
