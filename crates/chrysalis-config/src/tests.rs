//! Configuration tests.

use crate::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.validate().is_ok());
    assert!(config.platforms.web.flutter.run_pub_get);
    assert!(config.platforms.web.plugins.minify.enabled);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let yaml_str = serde_yaml::to_string(&config).unwrap();
    assert!(yaml_str.contains("project:"));
    assert!(yaml_str.contains("build:"));
    assert!(yaml_str.contains("minify_js:"));
}

#[test]
fn test_config_from_file() {
    let config_content = r#"
platforms:
  web:
    flutter:
      release: true
      run_pub_get: true

build:
  clean_before_build: true
  verbose: true
  parallel_jobs: 4

platforms:
  web:
    plugins:
      minify:
        enabled: true
        minify_js: true
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config = Config::from_file(temp_file.path()).unwrap();
    assert!(config.platforms.web.flutter.release);
    assert_eq!(config.build.parallel_jobs, 4);
    assert_eq!(config.build.verbose, true);
}

#[test]
fn test_config_builder() {
    let config = Config::builder()
        .with_build(BuildConfig {
            verbose: true,
            parallel_jobs: 2,
            ..Default::default()
        })
        .build();

    assert!(config.build.verbose);
    assert_eq!(config.build.parallel_jobs, 2);
}
