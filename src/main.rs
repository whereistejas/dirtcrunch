mod container;

use container::*;

#[tokio::main]
async fn main() {
    let mut mycontainer = MyContainer::new();

    let image_name = "airbyte/source-postgres".to_string();

    mycontainer.prepare_image(&image_name).await;

    mycontainer.prepare_container(&image_name).await;

    let result = mycontainer.start_container("spec").await;
}
