
/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anyhow::Error;
use camino::Utf8PathBuf as PathBuf;
use std::process::{Command, Output};
use std::result;
use std::sync::Once;


pub type Result<T> = result::Result<T, Error>;

pub const DOCKER_IMAGE: &str = "docker.io/debian:bullseye-slim";
pub const DOCKER_CMD: &str = "podman";

pub fn docker(cmd: Vec<&str>) -> Result<Output> {
    println!("CMD: {:?}", cmd);
    let out = Command::new(DOCKER_CMD)
        .args(cmd)
        .output()?;
    let stdout = String::from_utf8(out.clone().stdout).unwrap();
    let stderr = String::from_utf8(out.clone().stderr).unwrap();
    println!("STDOUT: {stdout}");
    println!("STDERR: {stderr}");
    assert!(out.status.success());
    Ok(out)
}

pub struct Container {
    id: String,
}

impl Container {
    pub fn new() -> Result<Self> {
        let running = docker(vec!["run", "--detach", DOCKER_IMAGE, "sleep", "15m"])?;
        let container = Container {
            id: String::from_utf8(running.stdout)?.trim().to_string()
        };
        Ok(container)
    }

    pub fn binary_path(&self, src_bin: &PathBuf) -> Result<PathBuf> {
        let bin = src_bin.components().last().unwrap();
        let dest_base = PathBuf::from("/usr/local/bin");
        let dest_bin = dest_base.join(bin);
        Ok(dest_bin)
    }

    pub fn copy_binary(&self, src_bin: &PathBuf) -> Result<PathBuf> {
        let dest_bin = self.binary_path(src_bin)?;

        let _out = self.cp(src_bin.as_str(), dest_bin.as_str())?;
        self.exec(vec!["chmod", "755", dest_bin.as_str()])?;

        Ok(dest_bin)
    }

    pub fn kill(&self) -> Result<()> {
        let _out = docker(vec!["rm", "--force", self.id.as_str()])?;
        Ok(())
    }

    pub fn exec(self: &Self, cmd: Vec<&str>) -> Result<Output> {
        self.exec_as("root", cmd)
    }

    pub fn exec_as(self: &Self, user: &str, cmd: Vec<&str>) -> Result<Output> {
        let out = Command::new(DOCKER_CMD)
            .arg("exec")
            .arg("--user").arg(user)
            .arg("-i")
            .arg(&self.id)
            .args(cmd)
            .output()?;
        Ok(out)
    }

    pub fn exec_w_pass<'a>(self: &Self, user: &'a str, pass: &'a str, mut cmd: Vec<&'a str>) -> Result<Output>
    {
        let mut ncmd = vec!["echo", pass, "|"];
        ncmd.append(&mut cmd);
        let out = self.exec_as(user, ncmd)?;
        Ok(out)
    }

    pub fn cp(self: &Self, from: &str, to: &str) -> Result<Output> {
        let remote = format!("{}:{}", self.id, to);
        let out = docker(vec!["cp", from, remote.as_str()])?;
        if !out.status.success() {
            anyhow::bail!("Copy of {} to {} failed", from, remote);
        }
        Ok(out)
    }

}

impl Drop for Container {
    fn drop(self: &mut Self) {
        self.kill().unwrap();
    }
}


static BUILD_LOCK: Once = Once::new();

// FIXME: Could merge this with Container, but not worth it ATM.
fn build_in_container(target_ext: &str, projdir: &str, features: &str, rust_ver: &str) -> Result<Output> {
     let build_image = format!("docker.io/rust:{rust_ver}-slim-bullseye");

    let builddir = "/tmp/src";
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
                          build_image.as_str()];

    let out = docker([docker_cli, cargo_cli].concat())?;

    Ok(out)
}

pub fn build_target(bin_name: &str, projdir: &str, features: Option<&str>, rust_ver: &str) -> Result<PathBuf> {
    let ext_base = "docker";
    let fstr = features.unwrap_or("");
    let target_ext = format!("{ext_base}/{}", fstr.replace(" ", "_"));

    BUILD_LOCK.call_once(|| { build_in_container(&target_ext, projdir, fstr, rust_ver).unwrap(); } );

    let bin = PathBuf::from(format!("{projdir}/target/{target_ext}/release/{bin_name}"));
    Ok(bin)
}
