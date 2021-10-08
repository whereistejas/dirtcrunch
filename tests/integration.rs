use dirtcrunch::{create_file, create_objects, get_specs};
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test() {
    let json = get_specs("airbyte/source-file").await;
    let objects = create_objects("SourceFile", "airbyte/source-file", json);
    let file = create_file(objects);

    let path = Path::new("src/source-file.rs");

    assert!(fs::write(path, &file).is_ok());
}
