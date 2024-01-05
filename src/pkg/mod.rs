use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

mod asset;
mod golang;
mod manifest;
mod npm;
mod pip;
pub mod version;

pub use asset::GithubRelease;
pub use golang::Go;
pub use manifest::{Entry, Manifest};
pub use npm::NPM;
pub use pip::PIP;
pub use version::Version;

#[derive(Deserialize, Serialize)]
pub enum PackageKind {
    #[serde(rename = "lspserver")]
    LSPServer,
    #[serde(rename = "linter")]
    Linter,
    #[serde(rename = "formatter")]
    Formatter,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

/// Represents a release.
pub struct Release {
    pub name: String,
    pub tag: String,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn try_get_version(&self) -> Result<Version> {
        let version = Version::try_from(&self.tag)?;
        Ok(version)
    }
}

/// Trait for getting releases of a package.
pub trait Releases {
    /// Get the latest release, if it exists.
    fn latest(&self) -> Result<Option<Release>>;

    /// Try get a release from tag name.
    fn get_from_tag(&self, tag: &str) -> Result<Option<Release>>;

    /// Require that a latest release exists.
    fn require_latest(&self) -> Result<Release> {
        match self.latest()? {
            Some(release) => Ok(release),
            None => bail!("failed to get latest release"),
        }
    }
}

pub struct Dirs {
    /// Directory to put executable files.
    pub bin_dir: PathBuf,
    /// Directory to put additional files required by the packages.
    pub pkg_dir: PathBuf,
}

pub struct PkgInfo {
    /// The GitHub repository.
    pub repo: String,
    /// Name of the package.
    pub name: String,
    /// The name of the go module, e.g golang.org/x/tools/cmd/goimports.
    pub mod_name: String,
    /// Name of the binary, e.g gopls
    pub bin_name: String,
}

#[macro_export]
macro_rules! pkg_args {
    ($repo:expr, $name:expr) => {
        $crate::pkg::PkgInfo {
            repo: $repo.to_string(),
            name: $name.to_string(),
            bin_name: $name.to_string(),
            mod_name: $name.to_string(),
        }
    };
    ($repo:expr, $name:expr, $mod:expr) => {
        $crate::pkg::PkgInfo {
            repo: $repo.to_string(),
            name: $name.to_string(),
            bin_name: $name.to_string(),
            mod_name: $mod.to_string(),
        }
    };
    ($repo:expr, $name:expr, $mod:expr, $bin:expr) => {
        $crate::pkg::PkgInfo {
            repo: $repo.to_string(),
            name: $name.to_string(),
            bin_name: $bin.to_string(),
            mod_name: $mod.to_string(),
        }
    };
}

/// Trait for downloading assets for e.g Github releases.
pub trait Assets {
    fn download(&self, asset: &Asset) -> Result<Vec<u8>>;
}

// Called after an asset has been downloaded.
pub type AssetCallback = dyn Fn(&PkgInfo, &Dirs, &Path) -> Result<()>;

pub trait Installer: Send + Sync {
    /// Install the package.
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<()>;

    /// Uninstalls the package.
    fn uninstall(&self, info: &PkgInfo, dirs: &Dirs) -> Result<()> {
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

pub enum CallbackOperation {
    Install,
    Uninstall,
    // Update,
}

// Optional callback after a package has been installed.
pub type PackageCallback = dyn Fn(CallbackOperation, &PkgInfo, &Dirs) -> Result<()>;

pub struct Package {
    info: PkgInfo,
    installer: Box<dyn Installer>,
    releases: Box<dyn Releases>,
}

unsafe impl Send for Package {}
unsafe impl Sync for Package {}

impl Package {
    pub fn new(info: PkgInfo, installer: Box<dyn Installer>, releases: Box<dyn Releases>) -> Self {
        Self {
            info,
            installer,
            releases,
        }
    }

    /// Gives the name of the package.
    pub fn name(&self) -> &String {
        &self.info.name
    }

    /// Gives the repo of the package.
    pub fn repo(&self) -> &String {
        &self.info.repo
    }

    pub fn latest(&self) -> Result<Option<Release>> {
        self.releases.latest()
    }

    pub fn install(&self, version: Option<Version>, dirs: &Dirs) -> Result<Version> {
        let release = self.try_get_release(version)?;
        self.install_release(release, dirs)
    }

    pub fn update(&self, version: Option<Version>, dirs: &Dirs) -> Result<Version> {
        let release = self.try_get_release(version)?;
        self.uninstall(dirs)?;
        self.install_release(release, dirs)
    }

    pub fn uninstall(&self, dirs: &Dirs) -> Result<()> {
        self.installer.uninstall(&self.info, dirs)?;
        Ok(())
    }

    fn try_get_release(&self, version: Option<Version>) -> Result<Option<Release>> {
        match &version {
            Some(v) => {
                let version = v.to_string();
                self.releases.get_from_tag(&version)
            }
            None => self.releases.latest(),
        }
    }

    fn install_release(&self, release: Option<Release>, dirs: &Dirs) -> Result<Version> {
        self.installer.install(&self.info, dirs, release.as_ref())?;

        match release {
            Some(r) => r.try_get_version(),
            None => Ok(Version::Unknown("unknown".to_string())),
        }
    }
}
