use futures::{Stream, TryStreamExt};
use shiplift::{tty::TtyChunk, ContainerOptions, Docker, PullOptions, Result, RmContainerOptions};

// This module implements methods that are used to interact with contianers.

#[derive(Clone)]
pub struct Container<'a> {
    docker: &'a Docker,
    container_id: String,
    image_name: String,
}

impl<'a> Container<'a> {
    pub fn new(docker: &'a Docker) -> Self {
        Self {
            docker,
            container_id: String::new(),
            image_name: String::new(),
        }
    }

    // Set the imagename that we need to pull from dokcerhub and use to create the container.
    pub fn imagename(&mut self, image: &str) {
        self.image_name = image.to_string();
    }

    // Pull image.
    pub async fn prepare_image(&mut self) {
        let images = self.docker.images();

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

    // Create the container with the given commands and volumes and return the ID of the new
    // container.
    async fn create_container(&mut self, command: Vec<&str>, volume: Option<Vec<&str>>) {
        // Create container
        let opts = match volume {
            Some(paths) => ContainerOptions::builder(&self.image_name)
                .attach_stdin(true)
                .attach_stdout(true)
                .cmd(command)
                .volumes(paths)
                .build(),
            None => ContainerOptions::builder(&self.image_name)
                .attach_stdin(true)
                .attach_stdout(true)
                .cmd(command)
                .build(),
        };

        let result = self
            .docker
            .containers()
            .create(&opts)
            .await
            .expect("Could not create the docker container in the current system.");

        // Get container ID.
        self.container_id = result.id.clone();
    }

    // Start container and read from its stdout for messages.
    pub async fn start_container(
        &mut self,
        command: Vec<&str>,
        volume: Option<Vec<&str>>,
    ) -> impl Stream<Item = Result<TtyChunk>> + 'a {
        self.create_container(command, volume).await;
        let container = self.docker.containers().get(&self.container_id);
        let (read, _) = container.attach().await.unwrap().split();

        // Start container
        container.start().await.expect("Failed to start container.");

        read
    }

    // Remove container and volumes.
    pub async fn delete_container(&mut self, volume: bool) -> Result<()> {
        let container = self.docker.containers().get(&self.container_id);
        let opts = RmContainerOptions::builder().volumes(volume).build();
        container.remove(opts).await
    }
}
