use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

mod asset;
mod cargo;
mod golang;
mod manifest;
mod npm;
mod pip;
pub mod version;

pub use asset::GithubRelease;
pub use cargo::Cargo;
pub use golang::Go;
pub use manifest::{Entry, Manifest};
pub use npm::NPM;
pub use pip::PIP;
pub use version::Version;

use crate::error::Error;

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
    /// Release assets, for instance binary files, archives etc.
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn try_get_version(&self) -> Result<Version> {
        let version = Version::try_from(&self.tag)?;
        Ok(version)
    }
}

pub struct Dirs {
    /// Root directory.
    pub root_dir: PathBuf,
    /// Directory to put executable files in.
    pub bin_dir: PathBuf,
    /// Directory to put additional files required by the packages.
    /// This is where downloaded files end up as well.
    pub pkg_dir: PathBuf,
}

pub struct PkgInfo {
    /// The GitHub repository.
    pub repo: String,
    /// Name of the package, that is, the name of installable target.
    /// It should not be confused with e.g. a pip or node package.
    pub name: String,
    /// The name of the module, e.g golang.org/x/tools/cmd/goimports.
    pub mod_name: String,
    /// Name of the binary, e.g gopls.
    pub bin_name: String,
}

/// pkg_args provides a more convenient way to
/// create a PkgInfo instance.
#[macro_export]
macro_rules! pkg_info {
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

/// An installer is able to install a given package (`info`)
/// to the correct directory.
pub trait Installer: Send + Sync {
    /// Install the package.
    fn install(&self, info: &PkgInfo, dirs: &Dirs, release: Option<&Release>) -> Result<(), Error>;

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

/// This signifies if a package was installed or removed.
pub enum CallbackOperation {
    Install,
    Uninstall,
}

/// Optional callback after a package has been installed.
pub type PackageCallback = dyn Fn(CallbackOperation, &PkgInfo, &Dirs) -> Result<()>;

/// Package contains the information for a package
/// as well as the ability to (un)install it.
pub struct Package {
    /// Basic information about the package.
    info: PkgInfo,
    /// If there exist an asset for the package this should
    /// be preferred since it's generally faster to install.
    asset_installer: Option<Box<dyn Installer>>,
    /// Installing a package using a package manager (e.g. cargo)
    /// is generally slower so this installer is not preferred
    /// over the asset installer.
    native_installer: Option<Box<dyn Installer>>,
}

unsafe impl Send for Package {}
unsafe impl Sync for Package {}

impl Package {
    pub fn new(
        info: PkgInfo,
        asset_installer: Option<Box<dyn Installer>>,
        native_installer: Option<Box<dyn Installer>>,
    ) -> Self {
        if asset_installer.is_none() && native_installer.is_none() {
            panic!(
                "invalid args for {}: at least one installer is required",
                info.name
            );
        }

        Self {
            info,
            asset_installer,
            native_installer,
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

    pub fn install(&self, release: Option<Release>, dirs: &Dirs) -> Result<Version> {
        let version = match &release {
            Some(r) => r.try_get_version(),
            None => Ok(Version::Unknown("unknown".to_string())),
        };

        if let Some(installer) = &self.asset_installer {
            log::info!("Trying to install {} from release asset", self.info.name);

            if let Err(err) = installer.install(&self.info, dirs, release.as_ref()) {
                match &err {
                    Error::MissingSystemAsset => {
                        log::info!("No asset found for system, checking if");
                    }
                    err => bail!("{}", err),
                }
            } else {
                log::info!("Succesfully installed {} for release asset", self.info.name);
                return version;
            }
        }

        if let Some(installer) = &self.native_installer {
            println!(
                "No release asset available for your system, trying a package manager instead."
            );

            if let Err(err) = installer.install(&self.info, dirs, release.as_ref()) {
                match &err {
                    Error::MissingProg(prog) => {
                        println!("Missing package manager for {}, {}", self.info.name, prog);
                    }
                    err => bail!("{}", err),
                }
            } else {
                log::info!("Succesfully installed {} for release asset", self.info.name);
                return version;
            }
        }

        bail!(
            "unable to install {}: no known installation method for your system",
            self.info.name
        )
    }

    pub fn update(&self, release: Option<Release>, dirs: &Dirs) -> Result<Version> {
        self.uninstall(dirs)?;
        self.install(release, dirs)
    }

    pub fn uninstall(&self, dirs: &Dirs) -> Result<()> {
        if let Some(installer) = &self.asset_installer {
            installer.uninstall(&self.info, dirs)?;
        } else if let Some(installer) = &self.native_installer {
            installer.uninstall(&self.info, dirs)?;
        }

        Ok(())
    }
}
