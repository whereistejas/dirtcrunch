use dirtcrunch::create_file;
use serde_yaml::from_str;
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
- sourceDefinitionId: b5ea17b1-f170-46dc-bc31-cc744ca984c1
  name: Microsoft SQL Server (MSSQL)
  dockerRepository: airbyte/source-mssql
  dockerImageTag: 0.3.6
  documentationUrl: https://docs.airbyte.io/integrations/sources/mssql
  icon: mssql.svg
  sourceType: database
"#;

    let path = env!("CARGO_TARGET_TMPDIR");
    println!("{:?}", path);

    let file = create_file(from_str(yaml).unwrap()).await;

    assert!(file.contains("struct AmazonSellerPartner"));
    assert!(file.contains("struct File"));
    assert!(file.contains("struct MicrosoftSQLServer"));

    let source_path = format!("{}/sources.rs", path);

    assert!(fs::write(source_path, &file).is_ok());
}
