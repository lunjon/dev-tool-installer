use super::{Dirs, Installer, PkgInfo, Release};
use crate::util;
use anyhow::Result;
use std::ffi::OsStr;
use std::process::{self, Stdio};

#[derive(Default)]
pub struct Go {}

unsafe impl Send for Go {}
unsafe impl Sync for Go {}

impl Installer for Go {
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<()> {
        util::require_command("go")?;
        let version = match release {
            Some(release) => release.try_get_version()?.to_string(),
            None => "latest".to_string(),
        };

        let mut cmd = new_cmd("go");
        cmd.env("GOBIN", &dirs.bin_dir);
        cmd.arg("install");
        cmd.arg(format!("{}@{}", info.mod_name, version));
        cmd.status()?;

        Ok(())
    }
}

fn new_cmd<S>(cmd: S) -> process::Command
where
    S: AsRef<OsStr>,
{
    let mut cmd = process::Command::new(cmd);
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd
}
