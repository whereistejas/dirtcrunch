use futures::{AsyncWriteExt, TryStreamExt};
use shiplift::{ContainerOptions, Docker, PullOptions};

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

    // TODO: This lifetime and return trait definition can be written in a better way.
    // use futures::{AsyncWriteExt, Stream, TryStreamExt};
    // use shiplift::{tty::TtyChunk, ContainerOptions, Docker, Error, PullOptions};
    // pub async fn start_container(
    //      &'_ mut self,
    //      command: &str,
    // ) -> impl futures::Stream<Item = Result<TtyChunk, Error>> + '_ {

    pub async fn start_container(&'_ mut self, command: &str) -> Vec<String> {
        // Create container
        let opts = ContainerOptions::builder(&self.image_name)
            .attach_stdin(true)
            .attach_stdout(true)
            .build();

        let result = self
            .docker
            .containers()
            .create(&opts)
            .await
            .expect("Could not create the docker container in the current system.");

        // Get container ID.
        self.container_id = result.id;

        let container = self.docker.containers().get(&self.container_id);
        let (read, mut write) = container.attach().await.unwrap().split();

        // Give connector command
        write
            .write(command.as_bytes())
            .await
            .expect("Failed to give connector command to docker container.");

        // Start container
        container
            .start()
            .await
            .expect("Failed to start discussion.");

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
