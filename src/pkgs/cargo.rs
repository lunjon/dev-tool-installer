use crate::pkg::{CargoInstaller, Package};
use crate::{config::Config, pkg_info};

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![
        package(cfg, "exa", "https://github.com/ogham/exa", "exa"),
        package(cfg, "fd", "https://github.com/sharkdp/fd", "fd-find"),
    ]
}

fn package(_cfg: &Config, name: &str, repo: &str, module: &str) -> Package {
    let args = pkg_info!(&repo, name, module);
    let installer = Box::new(CargoInstaller {});
    Package::new(args, None, Some(installer))
}
