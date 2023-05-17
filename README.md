# Docker Test Helper

This is a small helper library to build and run Rust applications inside a
Docker container (actually using podman). Its main use is to test applications
that require specific file paths or permissions that would be inappropriate to
perform locally on a development machine (e.g. `chmod` to root or enabling
`setuid`).

Right now it is somewhat specific to my use-cases, but may be of use to others.

```rust
use docker_test::{build::build_image_sync, util::build_and_deploy};

fn build_test_image() -> String {
    let dir = "tests/custom-container";
    let name = "docker-test-self-test-image";
    build_image_sync(dir, name).unwrap()
}

#[test]
fn is_root() {
    let image_name = build_test_image();
    let (container, bin) = build_and_deploy(
        "testproj",
        Some("tests/testproj"),
        None,
        image_name.as_str(),
    )
    .unwrap();
    let out = container.exec(vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker! My UID is [0]"));
}

#[test]
fn not_root() {
    let image_name = build_test_image();
    let (container, bin) = build_and_deploy(
        "testproj",
        Some("tests/testproj"),
        None,
        image_name.as_str(),
    )
    .unwrap();
    let out = container
        .exec_as("nobody", vec![bin.as_str(), "--help"])
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker! My UID is [65534]"));
}

#[test]
fn stock_image() {
    let image_name = "docker.io/rust:1.64.0-slim-bullseye";
    let (container, bin) = build_and_deploy(
        "testproj",
        Some("tests/testproj"),
        None,
        image_name,
    )
    .unwrap();
    let out = container
        .exec_as("nobody", vec![bin.as_str(), "--help"])
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker! My UID is [65534]"));
}
```
