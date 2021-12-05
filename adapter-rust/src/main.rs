use std::{env, process::Command};

use serde::Deserialize;
use shared::{Bench, BenchStats};

fn main() {
	let bench = run_benchmark().unwrap();
	println!("{}", serde_json::to_string(&bench).unwrap());
}

#[derive(Debug, Deserialize)]
struct CargoBench {
	#[serde(rename = "type")]
	_type: String,
	name: String,
	median: i64,
	deviation: i64,
}

enum BenchType {
	P1,
	P2,
	Parse,
}

fn run_benchmark() -> anyhow::Result<Bench> {
	let args: Vec<_> = env::args().collect();
	let command = if args.len() == 2 {
		let day: i64 = args[1].parse().unwrap();
		format!(
			"cargo bench -- day{:02} --format json --Z unstable-options",
			day
		)
	} else if args.len() == 3 {
		let executable = &args[1];
		let day: i64 = args[2].parse().unwrap();
		format!(
			"{} --bench day{:02} --format json --Z unstable-options",
			executable, day
		)
	} else {
		eprintln!("Usage: ... [EXECUTABLE] DAY ");
		panic!();
	};

	let output = Command::new("sh").arg("-c").arg(command).output()?;
	if !output.status.success() {
		anyhow::bail!("Failed to run benchmark");
	}

	let output = std::str::from_utf8(&output.stdout)?;
	let cargo_benches: Vec<_> = output
		.lines()
		.map(serde_json::from_str::<CargoBench>)
		.filter_map(Result::ok)
		.filter(|cargo_bench| cargo_bench._type == "bench")
		.collect();

	let mut bench = Bench::default();
	for cargo_bench in cargo_benches {
		let bench_type = match get_type(&cargo_bench.name) {
			Ok(bench_type) => bench_type,
			Err(_) => continue,
		};

		let stats = BenchStats {
			average_ns: cargo_bench.median,
			deviation_ns: cargo_bench.deviation,
		};

		match bench_type {
			BenchType::P1 => bench.p1 = Some(stats),
			BenchType::P2 => bench.p2 = Some(stats),
			BenchType::Parse => bench.parse = Some(stats),
		}
	}

	Ok(bench)
}

fn get_type(name: &str) -> anyhow::Result<BenchType> {
	if name.contains("part_one")
		|| name.contains("part_1")
		|| name.contains("part1")
		|| name.contains("p1")
	{
		return Ok(BenchType::P1);
	}

	if name.contains("part_two")
		|| name.contains("part_2")
		|| name.contains("part2")
		|| name.contains("p2")
	{
		return Ok(BenchType::P2);
	}

	if name.contains("parse") {
		return Ok(BenchType::Parse);
	}

	anyhow::bail!("Could not read bench type from name: {}", name)
}
