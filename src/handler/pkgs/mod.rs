use crate::github::GitHubClient;
use crate::pkg::{Go, Package, PIP};
use crate::{config::Config, pkg_args};
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::HashMap;

mod asset;
mod npm;

pub type Packages = HashMap<String, Package>;

#[derive(Deserialize)]
pub struct JsonFile {
    packages: Vec<JsonPackage>,
}

#[derive(Deserialize)]
pub struct JsonPackage {
    name: String,
    // kind: String,
    installer: String,
    repo: String,
    pkg: serde_json::Value,
}

#[derive(Deserialize)]
struct Pkg {
    module: String,
    deps: Vec<String>,
    bin: Option<String>,
}

#[derive(Deserialize)]
struct AssetPkgInfo {
    bin: Option<String>,
}

pub fn get_packages(cfg: &Config) -> Result<Packages> {
    let mut pkgs: Packages = HashMap::new();

    let mut insert = |name: &str, pkg: Package| {
        pkgs.insert(name.to_string(), pkg);
    };

    let packages_string = include_str!("packages.json");

    let js: JsonFile = serde_json::from_str(packages_string)?;
    for mut pkg in js.packages {
        pkg.repo = pkg
            .repo
            .trim_start_matches("https://github.com/")
            .to_string();

        match pkg.installer.as_str() {
            "go" => {
                let pkginfo: Pkg = serde_json::from_value(pkg.pkg)?;
                let bin = match pkginfo.bin {
                    Some(b) => b,
                    None => pkg.name.clone(),
                };

                let args = pkg_args!(&pkg.repo, pkg.name, pkginfo.module, bin);
                let installer = Box::<Go>::default();
                let gh = gh_client(cfg, &pkg.repo);
                let p = Package::new(args, installer, gh);
                insert(&pkg.name, p);
            }
            "pip" => {
                let pkginfo: Pkg = serde_json::from_value(pkg.pkg)?;
                let bin = match pkginfo.bin {
                    Some(b) => b,
                    None => pkg.name.clone(),
                };

                let args = pkg_args!(&pkg.repo, pkg.name, pkginfo.module, bin);
                let gh = gh_client(cfg, &pkg.repo);
                let installer = Box::new(PIP::new(pkginfo.deps));
                let p = Package::new(args, installer, gh);
                insert(&pkg.name, p);
            }
            "npm" => {
                let package = npm::build(cfg, &pkg)?;
                insert(&pkg.name, package);
            }
            "release-asset" => {
                let package = asset::build(cfg, &pkg)?;
                insert(&pkg.name, package);
            }
            _ => bail!("unknown installer: {}", pkg.installer),
        }
    }

    Ok(pkgs)
}

fn gh_client(cfg: &Config, repo: &str) -> Box<GitHubClient> {
    let repo = repo.trim_start_matches("https://github.com/");
    Box::new(GitHubClient::new(cfg, repo.to_string()))
}
