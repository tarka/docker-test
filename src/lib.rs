
/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anyhow::Error;
use camino::Utf8PathBuf as PathBuf;
use libc;
use std::env;
use std::process::{Command, Output};
use std::result;
use std::sync::Once;


pub type Result<T> = result::Result<T, Error>;

pub const BUILD_IMAGE: &str = "rust:1.58-slim-bullseye";
pub const DOCKER_IMAGE: &str = "debian:bullseye-slim";
pub const TESTUSER: &str = "testuser";
pub const TESTPASS: &str = "testpass";

pub fn docker(cmd: Vec<&str>) -> Result<Output> {
    let out = Command::new("docker")
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
    src_bin: PathBuf,
    dest_bin: PathBuf,
    id: String,
}

impl Container {
    pub fn new(src_bin: PathBuf) -> Result<Self> {
        let running = docker(vec!["run", "--detach", DOCKER_IMAGE, "sleep", "15m"])?;

        let bin = src_bin.components().last().unwrap();
        let dest_base: PathBuf = PathBuf::from("/usr/local/bin");
        let dest_bin = dest_base.join(bin);

        let container = Container {
            src_bin,
            dest_bin,
            id: String::from_utf8(running.stdout)?.trim().to_string()
        };

        Ok(container)
    }

    pub fn src_str(&self) -> &str {
        self.src_bin.as_str()
    }

    pub fn dest_str(&self) -> &str {
        self.dest_bin.as_str()
    }

    pub fn kill(&self) -> Result<()> {
        let _out = docker(vec!["rm", "--force", self.id.as_str()])?;
        Ok(())
    }

    pub fn exec(self: &Self, cmd: Vec<&str>) -> Result<Output> {
        self.exec_as("root", cmd)
    }

    pub fn exec_as(self: &Self, user: &str, cmd: Vec<&str>) -> Result<Output> {
        let out = Command::new("docker")
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
        Ok(out)
    }

}

impl Drop for Container {
    fn drop(self: &mut Self) {
        self.kill().unwrap();
    }
}

fn getids() -> (u32, u32) {
    unsafe { (libc::geteuid(), libc::getegid()) }
}


static BUILD_LOCK: Once = Once::new();

// FIXME: Could merge this with Container, but not worth it ATM.
fn build_in_container(target_ext: &str, features: &str) -> Result<Output> {
    // See https://hub.docker.com/_/rust
    // docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/usr/src/myapp -w /usr/src/myapp rust:1.23.0 cargo build --release

    let (uid, gid) = getids();
    let pwd = env::var("PWD")?;
    let builddir = "/usr/src";
    let target_base = format!("{builddir}/target");
    let imgtarget = format!("{target_base}/{target_ext}");
    let user = format!("{uid}:{gid}");
    let volume = format!("{pwd}:{builddir}");
    let cargo_env = format!("CARGO_HOME={target_base}/.cargo");

    let cargo_cli = vec!["cargo", "build", "--release",
                         "--features", features,
                         "--target-dir", imgtarget.as_str()];

    let docker_cli = vec!["run", "--rm",
                          "--user", user.as_str(),
                          "--volume", volume.as_str(),
                          "--workdir", builddir,
                          "--env", cargo_env.as_str(),
                          BUILD_IMAGE];

    let out = docker([docker_cli, cargo_cli].concat())?;

    Ok(out)
}

fn build_target(bin_name: &str, features: Option<&str>) -> Result<PathBuf> {
    let ext_base = "docker";
    let fstr = features.unwrap_or("");
    let target_ext = format!("{ext_base}/{}", fstr.replace(" ", "_"));

    BUILD_LOCK.call_once(|| { build_in_container(&target_ext, fstr).unwrap(); } );

    let bin = PathBuf::from(format!("target/{target_ext}/release/{bin_name}"));
    Ok(bin)
}

pub fn setup(bin_name: &str, features: Option<&str>) -> Result<Container> {
    let bin_path = build_target(bin_name, features)?;

    let container = Container::new(bin_path)?;

    // FIXME: move to new?
    container.cp(container.src_bin.as_str(), container.dest_str())?;
    container.exec(vec!["chown", "root.root", container.dest_str()])?;
    container.exec(vec!["chmod", "755", container.dest_str()])?;

    container.exec(vec!["adduser", "--disabled-password", TESTUSER])?;
    container.exec(vec!["echo", format!("{}\n{}\n", TESTPASS, TESTPASS).as_str(), "|", "passwd", TESTUSER])?;
    container.exec(vec!["addgroup", "--system", "sudoers"])?;

    Ok(container)
}
