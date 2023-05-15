
use std::sync::Once;

use docker_test::{util::build_and_deploy, build::build_image, Result};

static IMAGE_BUILD_LOCK: Once = Once::new();

fn build_test_image() -> Result<String> {
    let dir = "tests/custom-container";
    let name = "docker-test-self-test-image";
    IMAGE_BUILD_LOCK.call_once(|| build_image(dir, name).unwrap());
    Ok(name.to_string())
}

#[test]
fn build_run() {
    let image_name = build_test_image().unwrap();
    let (container, bin) = build_and_deploy("testproj", Some("tests/testproj"), None, image_name.as_str()).unwrap();
    let out = container.exec(vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}

#[test]
fn not_root() {
    let image_name = build_test_image().unwrap();
    let (container, bin) = build_and_deploy("testproj", Some("tests/testproj"), None, image_name.as_str()).unwrap();
    let out = container.exec_as("nobody", vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}
