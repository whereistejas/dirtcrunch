use dirtcrunch::{create_file, get_objects};
use serde_yaml::from_str;
use std::path::Path;
use std::{env, fs};

#[tokio::test]
async fn test() {
    let yaml = r#"
- sourceDefinitionId: e55879a8-0ef8-4557-abcf-ab34c53ec460
  name: Amazon Seller Partner
  dockerRepository: airbyte/source-amazon-seller-partner
  dockerImageTag: 0.2.0
  sourceType: api
  documentationUrl: https://docs.airbyte.io/integrations/sources/amazon-seller-partner
- sourceDefinitionId: d0243522-dccf-4978-8ba0-37ed47a0bdbf
  name: Asana
  dockerRepository: airbyte/source-asana
  dockerImageTag: 0.1.3
  documentationUrl: https://docs.airbyte.io/integrations/sources/asana
  sourceType: api
- sourceDefinitionId: 778daa7c-feaf-4db6-96f3-70fd645acc77
  name: File
  dockerRepository: airbyte/source-file
  dockerImageTag: 0.2.6
  documentationUrl: https://docs.airbyte.io/integrations/sources/file
  icon: file.svg
  sourceType: file
"#;

    let objects = get_objects(from_str(yaml).unwrap()).await;
    let file = create_file(objects);

    let tempdir = format!(
        "{}/target/tmp/_tempdir/",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    if !Path::new(&tempdir).exists() {
        fs::create_dir(&tempdir).expect("Failed to create the test directory")
    }

    let source_path = format!("{}/sources.rs", tempdir);
    assert!(fs::write(source_path, &file).is_ok());
}
