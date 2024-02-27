use super::{Dirs, Installer, PkgInfo, Release};
use crate::{error::Error, util};
use anyhow::Result;
use std::fs;

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
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<(), Error> {
        util::require_command("python")?;

        let name = match release {
            Some(r) => {
                let version = r.try_get_version()?;
                format!("{}=={}", info.mod_name, version)
            }
            None => info.mod_name.to_string(),
        };

        let target_dir = dirs.pkg_dir.join(&info.mod_name);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        // Create virtual env
        let venv_dir = target_dir.join("venv");
        if !venv_dir.exists() {
            let mut cmd = util::new_cmd("python");
            cmd.arg("-m");
            cmd.arg("venv");
            cmd.arg(&venv_dir);
            cmd.status()?;
        }

        // Install into virtual env
        let pip_path = venv_dir.join("bin").join("pip");
        let mut cmd = util::new_cmd(pip_path);
        cmd.args(["install", "--upgrade", &name]);
        cmd.args(&self.dependencies);
        util::run_cmd(&mut cmd)?;

        // Create symbolic link
        let link = dirs.bin_dir.join(&info.bin_name);
        let original = venv_dir.join("bin").join(&info.bin_name);
        util::symlink(&original, &link)?;

        Ok(())
    }
}
