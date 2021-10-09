use futures::{Stream, TryStreamExt};
use shiplift::{tty::TtyChunk, Docker, Result};
use shiplift::{ContainerOptions, PullOptions, RmContainerOptions};

/// The `Container` struct serves as the base for our interaction with the lower-level `shiplift`
/// crate that is used to interact with docker containers using the docker API.
#[derive(Clone)]
pub struct Container<'a> {
    docker: &'a Docker,
    container_id: String,
    image_name: String,
}

impl<'a> Container<'a> {
    /// Create a new instance of the `Container` object.
    pub fn new(docker: &'a Docker) -> Self {
        Self {
            docker,
            container_id: String::new(),
            image_name: String::new(),
        }
    }

    /// Set the image name. This is the image that will be pulled from dockerhub and used to create
    /// the containers.
    pub fn imagename(&mut self, image: &str) {
        self.image_name = image.to_string();
    }

    /// Pull the image set in the [`image_name`](image_name) field.
    pub async fn prepare_image(&mut self) {
        let images = self.docker.images();

        // Configure the Pull operation.
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

    /// Create the container with the given commands and volumes.
    async fn create_container(&mut self, command: Vec<&str>, volume: Option<Vec<&str>>) {
        // Configure the create opertion.
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

        // Save the container ID so that it can be used by the other methods.
        self.container_id = result.id;
    }

    /// Start container and return a stream to read from its stdout for messages.
    pub async fn start_container(
        &mut self,
        command: Vec<&str>,
        volume: Option<Vec<&str>>,
    ) -> impl Stream<Item = Result<TtyChunk>> + 'a {
        // Create the container.
        self.create_container(command, volume).await;

        let container = self.docker.containers().get(&self.container_id);
        // Create a read stream that will be used to read the output from the container.
        let (read, _) = container.attach().await.unwrap().split();

        // Start container
        container.start().await.expect("Failed to start container.");

        read
    }

    /// Remove container and volumes.
    pub async fn delete_container(&mut self, volume: bool) -> Result<()> {
        let container = self.docker.containers().get(&self.container_id);
        let opts = RmContainerOptions::builder().volumes(volume).build();
        container.remove(opts).await
    }
}
