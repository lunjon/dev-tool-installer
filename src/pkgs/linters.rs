use crate::config::Config;
use crate::pkg::{GoInstaller, Package};
use crate::pkg_info;

pub fn packages(_cfg: &Config) -> Vec<Package> {
    vec![actionlint()]
}

fn actionlint() -> Package {
    let args = pkg_info!(
        "https://github.com/rhysd/actionlint",
        "actionlint",
        "github.com/rhysd/actionlint/cmd/actionlint",
        "actionlint"
    );
    let installer = Box::new(GoInstaller {});
    Package::new(args, None, Some(installer))
}
