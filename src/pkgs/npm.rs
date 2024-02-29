use crate::config::Config;
use crate::pkg::{CallbackOperation, Dirs, NpmInstaller, Package, PkgInfo};
use crate::{pkg_info, util};
use std::fs;

pub fn packages(cfg: &Config) -> Vec<Package> {
    vec![
        package(
            cfg,
            "typescript-language-server",
            "https://github.com/typescript-language-server/typescript-language-server",
            vec!["typescript".to_string()],
        ),
        package(
            cfg,
            "pyright",
            "https://github.com/microsoft/pyright",
            vec![],
        ),
        package(
            cfg,
            "bash-language-server",
            "https://github.com/bash-lsp/bash-language-server",
            vec![],
        ),
        vscode_langservers_extracted(cfg),
    ]
}

fn vscode_langservers_extracted(_cfg: &Config) -> Package {
    let args = pkg_info!(
        "https://github.com/hrsh7th/vscode-langservers-extracted",
        "vscode-langservers-extracted"
    );

    let callback = Box::new(|op: CallbackOperation, info: &PkgInfo, dirs: &Dirs| {
        let bins = [
            "vscode-css-language-server",
            "vscode-html-language-server",
            "vscode-json-language-server",
            "vscode-markdown-language-server",
        ];

        match op {
            CallbackOperation::Install => {
                for bin_name in bins {
                    let pkg_bin = dirs.pkg_dir.join(&info.name).join("bin").join(bin_name);
                    let bin = dirs.bin_dir.join(bin_name);
                    util::symlink(&pkg_bin, &bin)?;
                }
            }
            CallbackOperation::Uninstall => {
                for bin_name in bins {
                    let bin = dirs.bin_dir.join(bin_name);
                    if bin.exists() {
                        fs::remove_file(&bin)?;
                    }
                }
            }
        }

        Ok(())
    });

    let installer = Box::new(NpmInstaller::new(vec![], callback));
    Package::new(args, None, Some(installer))
}

fn package(_cfg: &Config, name: &str, repo: &str, deps: Vec<String>) -> Package {
    let args = pkg_info!(repo, name);
    let callback = Box::new(|_op: CallbackOperation, _info: &PkgInfo, _dirs: &Dirs| Ok(()));
    let installer = Box::new(NpmInstaller::new(deps, callback));
    Package::new(args, None, Some(installer))
}
