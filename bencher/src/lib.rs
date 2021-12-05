use std::time::Duration;

use futures::StreamExt;
use shared::Bench;
use shiplift::{Container, ContainerOptions, Docker, LogsOptions, PullOptions, RmContainerOptions};
use tokio::time::sleep;

pub async fn run_benchmark(
	docker_image: &str,
	day: i64,
	aoc_session: &str,
) -> anyhow::Result<Bench> {
	pull_image(docker_image).await?;
	Ok(run_container(docker_image, day, aoc_session).await?)
}

async fn pull_image(docker_image: &str) -> anyhow::Result<()> {
	let docker = Docker::new();
	let mut stream = docker
		.images()
		.pull(&PullOptions::builder().image(docker_image).build());

	while let Some(pull_result) = stream.next().await {
		if pull_result.is_err() {
			anyhow::bail!("Failed to pull image {}", docker_image);
		}
	}

	Ok(())
}

async fn run_container(docker_image: &str, day: i64, aoc_session: &str) -> anyhow::Result<Bench> {
	let docker = Docker::new();

	let created_container = docker
		.containers()
		.create(
			&ContainerOptions::builder(docker_image)
				.env(&[format!("AOC_SESSION={}", aoc_session)])
				.cmd(vec![&format!("{}", day)])
				.build(),
		)
		.await?;

	let container = docker.containers().get(created_container.id);
	container.start().await?;

	for _ in 0..120 {
		sleep(Duration::from_secs(1)).await;
		let container_details = container.inspect().await?;
		let state = &container_details.state;
		if !state.running {
			if state.status == "exited" && state.exit_code == 0 {
				let logs = container_logs(&container).await?;
				let _ = container
					.remove(RmContainerOptions::builder().force(true).build())
					.await;
				return Ok(serde_json::from_str(&logs)?);
			} else {
				anyhow::bail!("Failed to run benchmarks in docker container");
			}
		}
	}

	let _ = container.stop(None).await;
	let _ = container
		.remove(RmContainerOptions::builder().force(true).build())
		.await;

	anyhow::bail!("Benchmark took too long");
}

async fn container_logs(container: &Container<'_>) -> anyhow::Result<String> {
	let mut stream = container.logs(&LogsOptions::builder().stdout(true).build());

	let mut logs = String::new();
	while let Some(res) = stream.next().await {
		match res {
			Ok(chunk) => match chunk {
				shiplift::tty::TtyChunk::StdOut(chunk) => logs += std::str::from_utf8(&chunk)?,
				_ => continue,
			},
			Err(e) => eprintln!("Error: {}", e),
		}
	}

	Ok(logs)
}
