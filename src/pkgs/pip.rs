use crate::pkg::{Package, PipInstaller};
use crate::{config::Config, pkg_info};

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![package(
        cfg,
        "pylsp",
        "https://github.com/python-lsp/python-lsp-server",
        "python-lsp-server",
    )]
}

fn package(_cfg: &Config, name: &str, repo: &str, module: &str) -> Package {
    let args = pkg_info!(&repo, name, module, name);
    let installer = Box::new(PipInstaller::new(vec![]));
    Package::new(args, None, Some(installer))
}
