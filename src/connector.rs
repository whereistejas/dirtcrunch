// TODO: doing this is probably not the best thing.
use crate::{container::Container, core_structs::AirbyteConnectionStatus};
use async_trait::async_trait;

#[derive(Debug)]
pub enum Command {
    Spec(String),
    Check(AirbyteConnectionStatus),
    Discover,
    Read,
}

#[async_trait]
pub trait Source {
    const IMAGE_NAME: &'static str;

    async fn specs() -> Command {
        let mut container = Container::new();
        container.prepare_image(Self::IMAGE_NAME).await;
        let result = container.start_container("spec").await;

        let search_string = "{\"type\": \"SPEC\",";
        let spec = result
            .iter()
            .find(|s| s.contains(search_string))
            .expect("Could not find specs.");
        // TODO: this needs to be re-written.
        let spec = spec.split_at(spec.find(search_string).unwrap());

        Command::Spec(spec.1.to_string())
    }
}
