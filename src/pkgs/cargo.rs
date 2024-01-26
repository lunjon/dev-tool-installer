use super::gh_client;
use crate::pkg::{Cargo, Package};
use crate::{config::Config, pkg_args};

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![
        package(cfg, "exa", "https://github.com/ogham/exa", "exa"),
        package(cfg, "fd", "https://github.com/sharkdp/fd", "fd-find"),
        package(cfg, "bat", "https://github.com/sharkdp/bat", "bat"),
        package(cfg, "just", "https://github.com/casey/just", "just"),
    ]
}

fn package(cfg: &Config, name: &str, repo: &str, module: &str) -> Package {
    let args = pkg_args!(&repo, name, module);
    let gh = gh_client(cfg, &args.repo);
    let installer = Box::new(Cargo {});
    Package::new(args, installer, gh)
}
