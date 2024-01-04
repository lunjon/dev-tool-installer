use super::{AssetCallback, Assets, Dirs, Installer, PkgInfo, Release};
use crate::util;
use anyhow::{bail, Result};
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
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<()> {
        if release.is_none() {
            bail!(
                "package {} requires a release to fetch assets from",
                info.name
            );
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
            None => bail!("failed to find release asset for {}", info.name),
        };

        let bytes = self.assets.download(asset)?;
        let targz = target_dir.join(&asset.name);
        util::write_file(&targz, &bytes)?;

        self.callback.as_ref()(info, dirs, &targz)
    }
}
