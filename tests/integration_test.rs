// use std::path::PathBuf;
// use std::process::Command;
// use anyhow::Result;
// use assert_fs::prelude::*;
// use predicates::prelude::*;
// use serde_json::json;
// use std::fs;

// fn setup_test_repo(temp: &assert_fs::TempDir) -> Result<()> {
//     // Create the basic structure
//     temp.child("pnpm-workspace.yaml").write_str("packages:\n  - 'packages/*'\n  - 'apps/*'")?;
    
//     temp.child("package.json").write_str(r#"{
//         "name": "sample-monorepo",
//         "version": "1.0.0",
//         "private": true
//     }"#)?;

//     // Create packages directory with shared libraries
//     let packages_dir = temp.child("packages");
//     fs::create_dir_all(packages_dir.path())?;

//     // Create logger package
//     let logger_dir = packages_dir.child("logger");
//     fs::create_dir_all(logger_dir.path())?;
//     logger_dir.child("package.json").write_str(r#"{
//         "name": "@sample/logger",
//         "version": "1.0.0",
//         "dependencies": {
//             "winston": "^3.11.0"
//         }
//     }"#)?;

//     // Create apps directory
//     let apps_dir = temp.child("apps");
//     fs::create_dir_all(apps_dir.path())?;

//     // Create API app
//     let api_dir = apps_dir.child("api");
//     fs::create_dir_all(api_dir.path())?;
//     api_dir.child("package.json").write_str(r#"{
//         "name": "@sample/api",
//         "version": "1.0.0",
//         "dependencies": {
//             "@sample/logger": "workspace:*"
//         }
//     }"#)?;

//     // Create Admin app
//     let admin_dir = apps_dir.child("admin");
//     fs::create_dir_all(admin_dir.path())?;
//     admin_dir.child("package.json").write_str(r#"{
//         "name": "@sample/admin",
//         "version": "1.0.0",
//         "dependencies": {
//             "@sample/types": "workspace:*"
//         }
//     }"#)?;

//     Ok(())
// }

// #[tokio::test]
// async fn test_bakehouse_against_sample_repo() -> Result<()> {
//     // Create a temporary directory for our test repo
//     let temp = assert_fs::TempDir::new()?;
//     setup_test_repo(&temp)?;

//     // Create output file in a separate directory
//     let output_dir = assert_fs::TempDir::new()?;
//     let output_file = output_dir.child("docker-bake.json");

//     // Run the bakehouse CLI
//     let status = Command::new(env!("CARGO_BIN_EXE_bakehouse"))
//         .args([
//             "--workspace",
//             temp.path().to_str().unwrap(),
//             "--output",
//             output_file.path().to_str().unwrap(),
//         ])
//         .status()?;

//     assert!(status.success(), "Bakehouse command failed");

//     // Verify the output file exists
//     output_file.assert(predicate::path::exists());

//     // Read and parse the generated bake file
//     let bake_content = std::fs::read_to_string(output_file.path())?;
//     let bake_file: serde_json::Value = serde_json::from_str(&bake_content)?;

//     // Verify the structure of the bake file
//     assert!(bake_file.is_object());
    
//     // Check for targets
//     let targets = bake_file.get("target").expect("Should have targets");
//     assert!(targets.is_object());

//     // Verify expected targets exist
//     let target_names: Vec<String> = targets.as_object().unwrap().keys().cloned().collect();
//     assert!(target_names.contains(&"@sample/api".to_string()));
//     assert!(target_names.contains(&"@sample/admin".to_string()));

//     // Check API target details
//     let api_target = &targets["@sample/api"];
//     assert_eq!(api_target["dockerfile"].as_str().unwrap(), "Dockerfile.bake");
//     assert!(api_target["context"].as_str().unwrap().contains("apps/api"));
    
//     // Verify dependencies
//     let api_deps = api_target["depends_on"].as_array().unwrap();
//     assert!(api_deps.contains(&json!("@sample/logger")));
//     assert!(api_deps.contains(&json!("@sample/config")));
//     assert!(api_deps.contains(&json!("@sample/types")));

//     // Check admin target details
//     let admin_target = &targets["@sample/admin"];
//     assert_eq!(admin_target["dockerfile"].as_str().unwrap(), "Dockerfile.bake");
//     assert!(admin_target["context"].as_str().unwrap().contains("packages/admin"));
    
//     // Verify admin dependencies
//     let admin_deps = admin_target["depends_on"].as_array().unwrap();
//     assert!(admin_deps.contains(&json!("@sample/types")));

//     // Check for default group
//     let groups = bake_file.get("group").expect("Should have groups");
//     let default_group = &groups["default"];
//     let default_targets = default_group["targets"].as_array().unwrap();
    
//     // Verify all targets are in the default group
//     for target_name in &target_names {
//         assert!(default_targets.contains(&json!(target_name)));
//     }

//     Ok(())
// } 