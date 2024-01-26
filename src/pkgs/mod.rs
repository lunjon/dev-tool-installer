use crate::config::Config;
use crate::github::GitHubClient;
use crate::pkg::Package;
use anyhow::Result;
use std::collections::HashMap;

mod asset;
mod cargo;
mod go;
mod npm;
mod pip;

pub type Packages = HashMap<String, Package>;

pub fn get_packages(cfg: &Config) -> Result<Packages> {
    let mut pkgs: Packages = HashMap::new();

    for pkg in go::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in cargo::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in npm::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in pip::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in asset::packages(cfg)? {
        pkgs.insert(pkg.name().to_string(), pkg);
    }

    Ok(pkgs)
}

fn gh_client(cfg: &Config, repo: &str) -> Box<GitHubClient> {
    let repo = repo.trim_start_matches("https://github.com/");
    Box::new(GitHubClient::new(cfg, repo.to_string()))
}
