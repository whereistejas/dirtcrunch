use futures::{AsyncWriteExt, TryStreamExt};
use shiplift::{ContainerOptions, Docker, PullOptions};

pub struct MyContainer {
    docker: Docker,
    container_id: String,
}

impl MyContainer {
    pub fn new() -> Self {
        MyContainer {
            docker: Docker::new(),
            container_id: String::new(),
        }
    }

    pub async fn prepare_image(&self, image: &str) {
        let images = self.docker.images();

        // Pull image
        let opts = PullOptions::builder().image(image).tag("latest").build();

        println!("Pulling");

        if let Ok(pull_result) = images.pull(&opts).try_collect::<Vec<_>>().await {
            println!("{:?}", pull_result);
        } else {
            panic!("Could not pull the latest docker images from the internet.");
        }
    }

    pub async fn prepare_container(&mut self, image: &str) {
        let containers = self.docker.containers();

        // Create container
        let opts = ContainerOptions::builder(&image)
            .attach_stdin(true)
            .attach_stdout(true)
            .build();

        let container_id = if let Ok(create_result) = containers.create(&opts).await {
            println!("{:?}", create_result.id);
            create_result.id
        } else {
            panic!("Could not create the docker container in the current system.");
        };

        // Get container ID.
        self.container_id = container_id;
    }

    pub async fn start_container(&mut self, command: &str) -> Vec<String> {
        let container = self.docker.containers().get(&self.container_id);
        let (read, mut write) = container.attach().await.unwrap().split();

        // Give connector command
        match write.write(command.as_bytes()).await {
            Ok(_) => {}
            Err(e) => panic!(
                "Failed to give connector command to docker container: {}",
                e
            ),
        }

        // Start container
        container.start().await.unwrap();

        let result_bytes = match read.try_collect::<Vec<_>>().await {
            Ok(result) => result,
            Err(e) => panic!("Failed to read command output from docker container: {}", e),
        };

        let result = result_bytes
            .iter()
            .map(|s| String::from_utf8(s.to_vec()).unwrap())
            .collect::<Vec<_>>();

        result
    }
}
