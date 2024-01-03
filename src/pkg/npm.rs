use super::{CallbackOperation, Dirs, Installer, PackageCallback, PkgInfo, Release};
use crate::util::{new_cmd, symlink};
use anyhow::Result;
use std::fs;

pub struct NPM {
    dependencies: Vec<String>,
    symlink: bool,
    callback: Box<PackageCallback>,
}

impl NPM {
    pub fn new(symlink: bool, dependencies: Vec<String>, callback: Box<PackageCallback>) -> Self {
        Self {
            dependencies,
            symlink,
            callback,
        }
    }
}

unsafe impl Send for NPM {}
unsafe impl Sync for NPM {}

impl Installer for NPM {
    fn install(&self, info: &PkgInfo, release: &Release, dirs: &Dirs) -> Result<()> {
        let version = release.try_get_version()?;

        let target_dir = dirs.pkg_dir.join(&info.mod_name);
        let node_modules = target_dir.join("node_modules");
        if !node_modules.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        let name = format!("{}@{}", info.mod_name, version);
        let mut cmd = new_cmd("npm");
        cmd.arg("install");
        cmd.arg("--global");
        cmd.arg("--prefix");
        cmd.arg(&target_dir);
        cmd.arg(name);
        cmd.args(&self.dependencies);
        cmd.status()?;

        if self.symlink {
            // Create symbolic link
            let link = dirs.bin_dir.join(&info.bin_name);
            let original = target_dir.join("bin").join(&info.bin_name);
            symlink(&original, &link)?;
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
