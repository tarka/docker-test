
use std::env;
use camino::{Utf8PathBuf as PathBuf};
use docker_test::{Container, Result, build_target};

fn setup(bin_name: &str, projdir: Option<&str>, features: Option<&str>, rust_ver: &str) -> Result<(Container, PathBuf)> {
    let pwd = env::var("PWD")?;
    let pd = if let Some(pd) = projdir {
        format!("{pwd}/{pd}")
    } else {
        pwd
    };

    let bin_path = build_target(bin_name, &pd, features, rust_ver)?;

    let container = Container::new()?;
    let dest_bin = container.copy_binary(&bin_path)?;

    Ok((container, dest_bin))
}


#[test]
fn build_run() {
    let (container, bin) = setup("testproj", Some("testproj"), None, "1.64.0").unwrap();
    let out = container.exec(vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}

#[test]
fn not_root() {
    let (container, bin) = setup("testproj", Some("testproj"), None, "1.64.0").unwrap();
    let out = container.exec_as("nobody", vec![bin.as_str(), "--help"]).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("Hello, docker!"));
}
