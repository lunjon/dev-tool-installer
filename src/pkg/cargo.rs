use super::{Dirs, Installer, PkgInfo, Release, Version};
use crate::{error::Error, util};
use anyhow::Result;
use std::process;

#[derive(Default)]
pub struct CargoInstaller {}

unsafe impl Send for CargoInstaller {}
unsafe impl Sync for CargoInstaller {}

impl Installer for CargoInstaller {
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<(), Error> {
        util::require_command("cargo")?;

        let mut cmd = process::Command::new("cargo");
        cmd.arg("install");
        cmd.arg("--root");
        cmd.arg(&dirs.root_dir);
        if let Some(release) = release {
            if let Ok(version) = release.try_get_version() {
                if let Version::Sem(maj, min, pat) = &version {
                    log::info!("Found semver version for {}: { }", &info.name, version);
                    cmd.arg("--version");
                    cmd.arg(format!("{}.{}.{}", maj, min, pat));
                } else {
                    log::info!(
                        "No semver found for {}, proceeding with latest version",
                        &info.name
                    );
                }
            }
        }
        cmd.arg(&info.mod_name);
        util::run_cmd(&mut cmd)?;

        Ok(())
    }
}
