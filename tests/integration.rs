use dirtcrunch::{create_file, get_specs};

#[tokio::test]
async fn test() {
    let json = get_specs("airbyte/source-file").await;
    let file = create_file("SourceFile", json);

    println!("{:#?}", file);
}
