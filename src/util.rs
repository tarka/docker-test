use camino::Utf8PathBuf as PathBuf;
use std::env;

use crate::{build::build_target, Container, Result};

// Builds a project in a rust container, then copies the resulting
// binary to a new running container.
pub fn build_and_deploy(
    bin_name: &str,
    projdir: Option<&str>,
    features: Option<&str>,
    image: &str,
) -> Result<(Container, PathBuf)> {
    let pwd = env::var("PWD")?;
    let pd = if let Some(pd) = projdir {
        format!("{pwd}/{pd}")
    } else {
        pwd
    };

    let bin_path = build_target(bin_name, &pd, features, image)?;

    let container = Container::new()?;
    let dest_bin = container.copy_binary(&bin_path)?;

    Ok((container, dest_bin))
}
