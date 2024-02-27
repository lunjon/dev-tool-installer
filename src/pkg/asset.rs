use super::{AssetCallback, Assets, Dirs, Installer, PkgInfo, Release};
use crate::{error::Error, util};
use anyhow::Result;
use regex::Regex;
use std::fs;

/// Used by packages installing from a github release assset.
pub struct GithubRelease {
    pattern: String,
    assets: Box<dyn Assets>,
    callback: Box<AssetCallback>,
}

impl GithubRelease {
    pub fn new(pattern: &str, assets: Box<dyn Assets>, callback: Box<AssetCallback>) -> Self {
        Self {
            assets,
            callback,
            pattern: pattern.to_string(),
        }
    }
}

unsafe impl Send for GithubRelease {}
unsafe impl Sync for GithubRelease {}

impl Installer for GithubRelease {
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<(), Error> {
        if release.is_none() {
            return Err(Error::MissingRelease);
        }

        let target_dir = dirs.pkg_dir.join(&info.mod_name);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        let regex = Regex::new(&self.pattern)?;
        let release = release.unwrap();
        let asset = release
            .assets
            .iter()
            .find(|asset| regex.is_match(&asset.name));

        let asset = match asset {
            Some(asset) => asset,
            None => {
                return Err(Error::MissingSystemAsset);
            }
        };

        let bytes = self.assets.download(asset)?;
        let targz = target_dir.join(&asset.name);
        util::write_file(&targz, &bytes)?;

        log::info!("Wrote tar.gz file to {:?}", &targz);

        if let Err(err) = self.callback.as_ref()(info, dirs, &targz) {
            log::error!("callback for {} failed: {}", info.name, err);
            Err(Error::Install {
                package: info.name.to_owned(),
                reason: format!("{}", err),
            })
        } else {
            Ok(())
        }
    }
}
