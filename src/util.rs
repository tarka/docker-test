
use std::env;
use camino::{Utf8PathBuf as PathBuf};

use crate::{Container, Result, build::build_target};

// Builds a project in a rust container, then copies the resulting
// binary to a new running container.
pub fn build_and_deploy(bin_name: &str, projdir: Option<&str>, features: Option<&str>, rust_ver: &str) -> Result<(Container, PathBuf)> {
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
