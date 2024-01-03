use super::{Dirs, Installer, PkgInfo, Release};
use crate::util::symlink;
use anyhow::Result;
use std::ffi::OsStr;
use std::fs;
use std::process::{self, Stdio};

pub struct PIP {
    dependencies: Vec<String>,
}

impl PIP {
    pub fn new(dependencies: Vec<String>) -> Self {
        Self { dependencies }
    }
}

unsafe impl Send for PIP {}
unsafe impl Sync for PIP {}

impl Installer for PIP {
    fn install(&self, info: &PkgInfo, release: &Release, dirs: &Dirs) -> Result<()> {
        let version = release.try_get_version()?;
        let name = format!("{}=={}", info.mod_name, version);

        let target_dir = dirs.pkg_dir.join(&info.mod_name);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        // Create virtual env
        let venv_dir = target_dir.join("venv");
        if !venv_dir.exists() {
            let mut cmd = new_cmd("python");
            cmd.arg("-m");
            cmd.arg("venv");
            cmd.arg(&venv_dir);
            cmd.status()?;
        }

        // Install into virtual env
        let pip_path = venv_dir.join("bin").join("pip");
        let mut cmd = new_cmd(pip_path);
        cmd.args(["install", "--upgrade", &name]);
        cmd.args(&self.dependencies);
        cmd.status()?;

        // Create symbolic link
        let link = dirs.bin_dir.join(&info.bin_name);
        let original = venv_dir.join("bin").join(&info.bin_name);
        symlink(&original, &link)?;

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
