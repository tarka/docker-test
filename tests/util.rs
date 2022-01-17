

use anyhow::Error;
use libc;
use std::env;
use std::process::{Command, Output};
use std::result;
use std::sync::Once;


pub type Result<T> = result::Result<T, Error>;

pub const BUILD_IMAGE: &str = "rust:1.58-slim-bullseye";
pub const DOCKER_IMAGE: &str = "debian:bullseye-slim";
pub const INST_BIN: &str = "/usr/bin/docker-test";
pub const TESTUSER: &str = "testuser";
pub const TESTPASS: &str = "testpass";

pub fn docker(cmd: Vec<&str>) -> Result<Output> {
    let out = Command::new("docker")
        .args(cmd)
        .output()?;
    assert!(out.status.success());
    Ok(out)
}

pub struct Container {
    id: String
}

impl Container {
    pub fn new() -> Result<Self> {
        let out = docker(vec!["run", "--detach", DOCKER_IMAGE, "sleep", "15m"])?;
        let docker = Container {
            id: String::from_utf8(out.stdout)?.trim().to_string()
        };

        Ok(docker)
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

// FIXME: Could merge this with Container if we split this into a
// crate, but not worth it ATM.
fn build_in_container(target_ext: &str) -> Result<Output> {
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
    let cli = vec!["run", "--rm",
                   "--user", user.as_str(),
                   "--volume", volume.as_str(),
                   "--workdir", builddir,
                   "--env", cargo_env.as_str(),
                   BUILD_IMAGE,
                   "cargo", "build", "--release", "--target-dir", imgtarget.as_str()];

    let out = docker(cli)?;

    Ok(out)
}

fn build_target(features: &str) -> Result<String> {
    let ext_base = "docker";
    let target_ext = if features == "" {
        format!("{ext_base}/default")
    } else {
        format!("{ext_base}/{}", features.replace(" ", "_"))
    };
    let bin = format!("target/{target_ext}/release/docker-test");

    BUILD_LOCK.call_once(|| { build_in_container(&target_ext).unwrap(); } );

    Ok(bin)
}

pub fn setup() -> Result<Container> {
    let bin = build_target("")?;

    let container = Container::new()?;
    container.exec(vec!["adduser", "--disabled-password", TESTUSER])?;
    container.exec(vec!["echo", format!("{}\n{}\n", TESTPASS, TESTPASS).as_str(), "|", "passwd", TESTUSER])?;
    container.exec(vec!["addgroup", "--system", "sudoers"])?;

    container.cp(bin.as_str(), INST_BIN)?;
    container.exec(vec!["chown", "root.root", INST_BIN])?;
    container.exec(vec!["chmod", "755", INST_BIN])?;

    Ok(container)
}
