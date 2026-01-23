//! Configuration tests.

use crate::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.validate().is_ok());
    assert!(config.flutter.run_pub_get);
    assert_eq!(config.build.chunk_size_kb, 400);
    assert!(config.plugins.minify.enabled);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("[flutter]"));
    assert!(toml_str.contains("[build]"));
    assert!(toml_str.contains("[plugins.minify]"));
    assert!(toml_str.contains("[plugins.hash]"));
}

#[test]
fn test_config_from_file() {
    let config_content = r#"
[flutter]
release = true
run_pub_get = true
target_dir = "build/web"

[build]
chunk_size_kb = 500
hash_length = 10

[plugins.minify]
enabled = true
minify_js = true
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path()).unwrap();
    assert!(config.flutter.release);
    assert_eq!(config.build.chunk_size_kb, 500);
    assert_eq!(config.build.hash_length, 10);
}

#[test]
fn test_config_builder() {
    let config = Config::builder()
        .flutter(FlutterConfig {
            release: false,
            ..Default::default()
        })
        .with_build(BuildConfig {
            chunk_size_kb: 300,
            ..Default::default()
        })
        .build();

    assert!(!config.flutter.release);
    assert_eq!(config.build.chunk_size_kb, 300);
}
