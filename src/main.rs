use futures::{prelude::*, TryStreamExt};
use shiplift::{tty::TtyChunk, ContainerOptions, Docker, PullOptions};

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let containers = docker.containers();
    let images = docker.images();

    let image_name = "airbyte/source-postgres".to_string();

    // Pull image
    let opts = PullOptions::builder()
        .image(&image_name)
        .tag("latest")
        .build();

    println!("Pulling");

    if let Ok(pull_result) = images.pull(&opts).try_collect::<Vec<_>>().await {
        println!("{:?}", pull_result);
    } else {
        panic!("Could not pull the latest docker images from the internet.");
    }

    // Create container
    let opts = ContainerOptions::builder(&image_name)
        .attach_stdin(true)
        .attach_stdout(true)
        .cmd(vec!["spec"])
        .build();

    let container_id = if let Ok(create_result) = containers.create(&opts).await {
        println!("{:?}", create_result.id);
        create_result.id
    } else {
        panic!("Could not create the docker container in the current system.");
    };

    // Get container ID.
    let container = containers.get(&container_id);

    let (read, mut write) = container.attach().await.unwrap().split();

    // Give connector command
    match write.write("spec".as_bytes()).await {
        Ok(_) => {}
        Err(e) => panic!("Failed to give connector command to docker container.", e),
    }

    // Start container
    container.start().await.unwrap();

    match read.try_collect::<Vec<_>>().await {
        Ok(spec_result) => print_chunk(spec_result),
        Err(e) => panic!("Failed to read connector spec from docker container.", e),
    }
}

fn print_chunk(chunks: Vec<TtyChunk>) {
    for chunk in chunks {
        match chunk {
            TtyChunk::StdOut(bytes) => println!("Stdout: {}", std::str::from_utf8(&bytes).unwrap()),
            TtyChunk::StdErr(bytes) => {
                eprintln!("Stdout: {}", std::str::from_utf8(&bytes).unwrap())
            }
            TtyChunk::StdIn(_) => unreachable!(),
        }
    }
}
