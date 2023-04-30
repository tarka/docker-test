# Docker Test Helper

This is a small helper library to build and run Rust applications inside a
Docker container (using podman). Its main use is to test applications that
require specific file paths or permissions that would be inappropriate to
perform locally on a development machine (e.g. `chmod` to root or enabling
`setuid`).

Right now it is somewhat specific to my use-cases, but may be of use to others.

```rust
use docker_test::util::build_and_deploy;

const RUST_VERSION: &str = "1.65.0";
const TEST_USER: &str = "nobody";

#[test]
fn help() {
    let (container, bin) = build_and_deploy("mybin", None, None, RUST_VERSION).unwrap();
    let out = container.exec(vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Run commands as a user"));
}

#[test]
fn not_root() {
    let (container, bin) = build_and_deploy("mybin", None, None, RUST_VERSION).unwrap();
    let out = container.exec_as(TEST_USER, vec![bin.as_str(), "/bin/ls"]).unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap()
            .contains("Error: Not running as root"));
}
```
