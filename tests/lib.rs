
use docker_test as dt;

#[test]
fn help() {
    let container = dt::setup("testproj", Some("testproj"), None).unwrap();
    let out = container.exec(vec![container.dest_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}