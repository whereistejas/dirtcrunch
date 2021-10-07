use dirtcrunch::{create_file, get_specs};
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test() {
    let json = get_specs("airbyte/source-file").await;
    let file = create_file("SourceFile", json);

    let path = Path::new("src/source-file.rs");

    assert!(fs::write(path, &file).is_ok());
}
