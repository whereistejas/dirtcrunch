use dirtcrunch::*;

// This test is basically an example of how to use this crate.

struct File {}

impl Source for File {
    const CONNECTOR: &'static str = "airbyte/source-file";
}

#[tokio::test]
async fn test() {
    let result = File::specs().await;
    println!("{:?}", result);

    assert!(false);
}
