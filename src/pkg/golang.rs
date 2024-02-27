use super::{Dirs, Installer, PkgInfo, Release};
use crate::{error::Error, util};
use anyhow::Result;

#[derive(Default)]
pub struct Go {}

unsafe impl Send for Go {}
unsafe impl Sync for Go {}

impl Installer for Go {
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<(), Error> {
        util::require_command("go")?;

        let version = match release {
            Some(release) => release.try_get_version()?.to_string(),
            None => "latest".to_string(),
        };

        let mut cmd = util::new_cmd("go");
        cmd.env("GOBIN", &dirs.bin_dir);
        cmd.arg("install");
        cmd.arg(format!("{}@{}", info.mod_name, version));
        cmd.status()?;

        util::run_cmd(&mut cmd)?;
        Ok(())
    }
}
