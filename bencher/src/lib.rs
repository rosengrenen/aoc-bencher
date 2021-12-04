use std::collections::HashMap;

use futures::StreamExt;
use shared::Bench;
use shiplift::{ContainerOptions, Docker, PullOptions};

pub async fn run_benchmark(user: &str, docker_image: &str, day: i64) -> anyhow::Result<Bench> {
	pull_image(docker_image).await?;
	Ok(run_container(docker_image, day).await?)
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

async fn run_container(docker_image: &str, day: i64) -> anyhow::Result<Bench> {
	let docker = Docker::new();

	let lmao = docker
		.containers()
		.create(
			&ContainerOptions::builder(docker_image)
				.cmd(vec![&format!("{}", day)])
				.build(),
		)
		.await;

	let lmao = lmao.unwrap();
	println!("{:?}", lmao);

	// let opts = ExecContainerOptions::builder()
	// 	.cmd(cmd.iter().map(String::as_str).collect())
	// 	.attach_stdout(true)
	// 	.attach_stderr(true)
	// 	.build();

	// let exec = Exec::create(&docker, &id, &opts).await.unwrap();

	// println!("{:#?}", exec.inspect().await.unwrap());

	// let mut stream = exec.start();

	// stream.next().await;

	// println!("{:#?}", exec.inspect().await.unwrap());

	anyhow::bail!("kekw");
}
