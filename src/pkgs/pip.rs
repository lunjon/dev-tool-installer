use crate::pkg::{Package, PIP};
use crate::{config::Config, pkg_args};

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![package(
        cfg,
        "pylsp",
        "https://github.com/python-lsp/python-lsp-server",
        "python-lsp-server",
    )]
}

fn package(_cfg: &Config, name: &str, repo: &str, module: &str) -> Package {
    let args = pkg_args!(&repo, name, module, name);
    let installer = Box::new(PIP::new(vec![]));
    Package::new(args, installer)
}
