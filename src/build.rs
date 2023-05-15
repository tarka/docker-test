
use std::process::Output;
use std::sync::Once;
use anyhow::anyhow;
use camino::Utf8PathBuf as PathBuf;
use once_cell::sync::Lazy;
use quick_cache::{sync::Cache, GuardResult};

use crate::{cmd, Result};


// FIXME: Could merge this with Container, but not worth it ATM.
pub fn build_in_container(target_ext: &str, projdir: &str, features: &str, image: &str) -> Result<Output> {
    let builddir = "/opt/src";
    let target_base = format!("{builddir}/target");
    let imgtarget = format!("{target_base}/{target_ext}");
    let volume = format!("{projdir}:{builddir}");
    let cargo_env = format!("CARGO_HOME={target_base}/.cargo");

    let cargo_cli = vec!["cargo", "build", "--release",
                         "--features", features,
                         "--target-dir", imgtarget.as_str()];

    let docker_cli = vec!["run", "--rm",
                          "--volume", volume.as_str(),
                          "--workdir", builddir,
                          "--env", cargo_env.as_str(),
                          image];

    let out = cmd([docker_cli, cargo_cli].concat())?;

    Ok(out)
}


static APP_BUILD_LOCK: Once = Once::new();

// Build a project in a rust container. Uses locking to ensure
// concurrent test runs share a common build.
pub fn build_target(bin_name: &str, projdir: &str, features: Option<&str>, image: &str) -> Result<PathBuf> {
    let ext_base = "docker";
    let fstr = features.unwrap_or("");
    let target_ext = format!("{ext_base}/{}", fstr.replace(" ", "_"));

    APP_BUILD_LOCK.call_once(|| { build_in_container(&target_ext, projdir, fstr, image).unwrap(); } );

    let bin = PathBuf::from(format!("{projdir}/target/{target_ext}/release/{bin_name}"));
    Ok(bin)
}

pub fn build_image(dir: &str, name: &str) -> Result<String> {
    let cli = vec!["build", "--tag", name, dir];

    let out = cmd(cli)?;
    let stdout = String::from_utf8(out.stdout)?;
    let id = stdout.lines()
        .last()
        .ok_or(anyhow!("No output from build command"))?;

    Ok(id.to_string())
}


static IMAGE_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| Cache::new(16) );

pub fn build_image_sync(dir: &str, name: &str) -> Result<String> {
    let key = format!("{}-{}", dir, name);
    match IMAGE_CACHE.get_value_or_guard(&key, None) {
        GuardResult::Timeout => Err(anyhow!("Unexpected timeout building base image")),
        GuardResult::Value(val) => Ok(val),
        GuardResult::Guard(guard) => {
            let id = build_image(dir, name)?;
            guard.insert(id.clone());
            Ok(id)
        }
    }
}
