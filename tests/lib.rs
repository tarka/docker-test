
use docker_test::util::build_and_deploy;

#[test]
fn build_run() {
    let (container, bin) = build_and_deploy("testproj", Some("tests/testproj"), None, "1.64.0").unwrap();
    let out = container.exec(vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}

#[test]
fn not_root() {
    let (container, bin) = build_and_deploy("testproj", Some("tests/testproj"), None, "1.64.0").unwrap();
    let out = container.exec_as("nobody", vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}
