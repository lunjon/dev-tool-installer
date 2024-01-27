use super::{CallbackOperation, Dirs, Installer, PackageCallback, PkgInfo, Release};
use crate::util;
use anyhow::Result;
use std::fs;

pub struct NPM {
    dependencies: Vec<String>,
    callback: Box<PackageCallback>,
}

impl NPM {
    pub fn new(dependencies: Vec<String>, callback: Box<PackageCallback>) -> Self {
        Self {
            dependencies,
            callback,
        }
    }
}

unsafe impl Send for NPM {}
unsafe impl Sync for NPM {}

impl Installer for NPM {
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<()> {
        util::require_command("npm")?;

        let name = match release {
            Some(r) => {
                let version = r.try_get_version()?;
                format!("{}@{}", info.mod_name, version)
            }
            None => info.mod_name.to_string(),
        };

        let target_dir = dirs.pkg_dir.join(&info.mod_name);
        let node_modules = target_dir.join("node_modules");
        if !node_modules.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        let mut cmd = util::new_cmd("npm");
        cmd.arg("install");
        cmd.arg("--global");
        cmd.arg("--prefix");
        cmd.arg(&target_dir);
        cmd.arg(name);
        cmd.args(&self.dependencies);
        util::run_cmd(&mut cmd)?;

        // Create symbolic link if there exists a bin in pkg dir
        let original = target_dir.join("bin").join(&info.bin_name);
        if original.exists() {
            let link = dirs.bin_dir.join(&info.bin_name);
            util::symlink(&original, &link)?;
        }

        self.callback.as_ref()(CallbackOperation::Install, info, dirs)?;

        Ok(())
    }

    fn uninstall(&self, info: &PkgInfo, dirs: &Dirs) -> Result<()> {
        self.callback.as_ref()(CallbackOperation::Uninstall, info, dirs)?;

        let bin = dirs.bin_dir.join(&info.bin_name);
        if bin.exists() {
            fs::remove_file(&bin)?;
        }

        let pkg = dirs.pkg_dir.join(&info.mod_name);
        if pkg.exists() {
            fs::remove_dir_all(&pkg)?;
        }

        Ok(())
    }
}
