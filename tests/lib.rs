use docker_test::{build::build_image_sync, util::build_and_deploy};

fn build_test_image() -> String {
    let dir = "tests/custom-container";
    let name = "docker-test-self-test-image";
    build_image_sync(dir, name).unwrap()
}

#[test]
fn build_run() {
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
    assert!(stdout.contains("Hello, docker!"));
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
