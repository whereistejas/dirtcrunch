use futures::TryStreamExt;
use shiplift::{ContainerOptions, Docker, PullOptions};

// This module implements methods that are used to interact with contianers.

pub struct Container {
    docker: Docker,
    container_id: String,
    image_name: String,
}

impl Container {
    pub fn new() -> Self {
        Self {
            docker: Docker::new(),
            container_id: String::new(),
            image_name: String::new(),
        }
    }

    pub async fn prepare_image(&mut self, image: &str) {
        let images = self.docker.images();
        self.image_name = image.to_string();

        // Pull image
        let opts = PullOptions::builder()
            .image(&self.image_name)
            .tag("latest")
            .build();

        images
            .pull(&opts)
            .try_collect::<Vec<_>>()
            .await
            .expect("Could not pull the latest docker images from the internet.");
    }

    async fn create_container(&mut self, command: &str) {
        // Create container
        let opts = ContainerOptions::builder(&self.image_name)
            .attach_stdin(true)
            .attach_stdout(true)
            .cmd(vec![command])
            .build();

        let result = self
            .docker
            .containers()
            .create(&opts)
            .await
            .expect("Could not create the docker container in the current system.");

        // Get container ID.
        self.container_id = result.id;
    }

    // TODO: This lifetime and return trait definition can be written in a better way.
    // use futures::{AsyncWriteExt, Stream, TryStreamExt};
    // use shiplift::{tty::TtyChunk, ContainerOptions, Docker, Error, PullOptions};
    // pub async fn start_container(
    //      &'_ mut self,
    //      command: &str,
    // ) -> impl futures::Stream<Item = Result<TtyChunk, Error>> + '_ {

    pub async fn start_container(&'_ mut self, command: &str) -> Vec<String> {
        self.create_container(command).await;

        let container = self.docker.containers().get(&self.container_id);
        let (read, _) = container.attach().await.unwrap().split();

        // Start container
        container.start().await.expect("Failed to start container.");

        let result_bytes = read
            .try_collect::<Vec<_>>()
            .await
            .expect("Failed to read command output from docker container.");

        result_bytes
            .iter()
            .map(|s| String::from_utf8(s.to_vec()).unwrap())
            .collect::<Vec<_>>()
    }
}
