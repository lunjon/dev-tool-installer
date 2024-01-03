use crate::{pkg::Version, util};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

/// Optional GitHub OAuth application
/// that can be used for authentication
/// in requests to GitHub API.
#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    #[serde(rename = "client-id")]
    pub client_id: String,
    #[serde(rename = "client-secret")]
    pub client_secret: String,
}

#[derive(Default, Deserialize, Serialize)]
pub struct PackageConfig {
    pub version: Option<Version>,
}

type PackageConfigs = HashMap<String, PackageConfig>;

#[derive(Default)]
pub struct Config {
    pub ensure_installed: Option<Vec<String>>,
    pub package_configs: PackageConfigs,
    pub auth: Option<Auth>,
}

impl Config {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        let file_config = if path.exists() {
            let config: FileConfig = util::toml_from_file(path)?;
            config
        } else {
            FileConfig::default()
        };

        let ensure_installed: Option<Vec<String>>;
        let package_configs: PackageConfigs;

        if let Some(pkgs) = file_config.packages {
            ensure_installed = pkgs.ensure_installed;
            package_configs = match pkgs.config {
                Some(cfg) => cfg,
                None => PackageConfigs::new(),
            };
        } else {
            ensure_installed = None;
            package_configs = PackageConfigs::new();
        }

        Ok(Self {
            ensure_installed,
            package_configs,
            auth: file_config.auth,
        })
    }
}

#[derive(Default, Deserialize, Serialize)]
struct Packages {
    #[serde(rename = "ensure-installed")]
    ensure_installed: Option<Vec<String>>,
    config: Option<PackageConfigs>,
}

#[derive(Default, Deserialize, Serialize)]
struct FileConfig {
    packages: Option<Packages>,
    auth: Option<Auth>,
}
