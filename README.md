# Docker Test Helper

This is a small helper library to build and run Rust applications inside a
Docker container. Its main use is to test applications that require specific
file paths or permissions that would be inappropriate to perform locally on a
development machine (e.g. `chmod` to root or enabling `setuid`).

Right now it is somewhat specific to my use-cases, but may be of use to others.

```rust

use docker_test::*;


#[test]
fn help() {
    let container = setup("rsu", None).unwrap();
    let out = container.exec(vec![container.dest_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Run commands as a user"));
}

#[test]
fn not_root() {
    let container = setup("rsu", None).unwrap();
    let out = container.exec_as(TESTUSER, vec![container.dest_str(), "/bin/ls"]).unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap()
            .contains("Error: Not running as root"));
}

```

See https://github.com/tarka/rsu/blob/main/tests/lib.rs for an example of use.
