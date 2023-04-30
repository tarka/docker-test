# Docker Test Helper

This is a small helper library to build and run Rust applications inside a
Docker container (using podman). Its main use is to test applications that
require specific file paths or permissions that would be inappropriate to
perform locally on a development machine (e.g. `chmod` to root or enabling
`setuid`).

Right now it is somewhat specific to my use-cases, but may be of use to others.

```rust
use docker_test as dt;

#[test]
fn help() {
    let container = dt::setup("rsu", None, None, "1.64.0").unwrap();
    let out = container.exec(vec![container.dest_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Run commands as a user"));
}

#[test]
fn not_root() {
    let container = dt::setup("rsu", None, None, "1.64.0").unwrap();
    let out = container.exec_as(dt::TESTUSER, vec![container.dest_str(), "/bin/ls"]).unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap()
            .contains("Error: Not running as root"));
}
```
