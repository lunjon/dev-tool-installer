use crate::pkg::{Go, Package};
use crate::{config::Config, pkg_info};

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![
        package(
            cfg,
            "gopls",
            "https://github.com/golang/tools",
            "golang.org/x/tools/gopls",
        ),
        package(
            cfg,
            "goimports",
            "https://github.com/golang/tools",
            "golang.org/x/tools/cmd/goimports",
        ),
        package(
            cfg,
            "lazygit",
            "https://github.com/jesseduffield/lazygit",
            "github.com/jesseduffield/lazygit",
        ),
        package(
            cfg,
            "actionlint",
            "https://github.com/rhysd/actionlint",
            "github.com/rhysd/actionlint/cmd/actionlint",
        ),
    ]
}

fn package(_cfg: &Config, name: &str, repo: &str, module: &str) -> Package {
    let args = pkg_info!(&repo, name, module, name);
    let installer = Box::new(Go {});
    Package::new(args, None, Some(installer))
}
