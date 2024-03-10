use crate::config::Config;
use crate::github::GitHubClient;
use crate::pkg::Package;
use anyhow::Result;
use std::collections::HashMap;

mod langservers;
mod linters;
mod misc;

pub type Packages = HashMap<String, Package>;

pub fn get_packages(cfg: &Config) -> Result<Packages> {
    let mut pkgs: Packages = HashMap::new();

    for pkg in langservers::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in misc::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }
    for pkg in linters::packages(cfg) {
        pkgs.insert(pkg.name().to_string(), pkg);
    }

    Ok(pkgs)
}

fn gh_client(cfg: &Config) -> Box<GitHubClient> {
    Box::new(GitHubClient::new(cfg))
}
