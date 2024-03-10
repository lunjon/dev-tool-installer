use super::gh_client;
use crate::config::Config;
use crate::pkg::{
    CallbackOperation, Dirs, GithubReleaseInstaller, GoInstaller, NpmInstaller, Package,
    PipInstaller, PkgInfo,
};
use crate::{pkg_info, util};
use std::fs;
use std::path::Path;

pub fn packages(cfg: &Config) -> Vec<Package> {
    let mut packages = vec![
        gopls(),
        elixir_ls(cfg),
        vscode_langservers_extracted(cfg),
        typescript_ls(),
        pyright(),
        bash_ls(),
        pylsp(),
    ];

    let maybe_packages = vec![rust_analyzer(cfg), clojure_lsp(cfg)];
    packages.extend(maybe_packages.into_iter().flatten());

    packages
}

fn gopls() -> Package {
    let args = pkg_info!(
        "https://github.com/golang/tools",
        "gopls",
        "golang.org/x/tools/gopls",
        "gopls"
    );
    let installer = Box::new(GoInstaller {});
    Package::new(args, None, Some(installer))
}

fn elixir_ls(cfg: &Config) -> Package {
    let repo = "https://github.com/elixir-lsp/elixir-ls";
    let info = pkg_info!(&repo, "elixir-ls");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);

        util::decompress(path, &pkg_dir)?;

        let executable = pkg_dir.join("language_server.sh");
        let bin = dirs.bin_dir.join(&info.name);
        util::symlink(&executable, &bin)?;

        Ok(())
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            "^elixir-ls-v.*\\.zip$".to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        None,
    )
}

fn rust_analyzer(cfg: &Config) -> Option<Package> {
    let repo = "https://github.com/rust-lang/rust-analyzer";
    let args = pkg_info!(&repo, "rust-analyzer");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let bin = dirs.bin_dir.join(&info.name);
        util::decompress(path, &bin)?;
        util::make_executable(&bin)?;
        Ok(())
    };

    let asset_regex = if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "musl"
    )) {
        Some("rust-analyzer-x86_64-unknown-linux-musl.gz")
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        Some("rust-analyzer-x86_64-unknown-linux-gnu.gz")
    } else {
        None
    };

    asset_regex.map(|pattern| {
        Package::new(
            args,
            Some(Box::new(GithubReleaseInstaller::new(
                pattern.to_string(),
                gh_client(cfg),
                Box::new(callback),
            ))),
            None,
        )
    })
}

fn clojure_lsp(cfg: &Config) -> Option<Package> {
    let repo = "https://github.com/clojure-lsp/clojure-lsp";
    let args = pkg_info!(&repo, "clojure-lsp");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        let pkg_bin = pkg_dir.join(&info.bin_name);
        let bin = dirs.bin_dir.join(&info.bin_name);
        util::symlink(&pkg_bin, &bin)
    };

    let asset_regex = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("clojure-lsp-native-linux-amd64.zip")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("clojure-lsp-native-linux-aarch64.zip")
    } else {
        None
    };

    asset_regex.map(|pattern| {
        Package::new(
            args,
            Some(Box::new(GithubReleaseInstaller::new(
                pattern.to_string(),
                gh_client(cfg),
                Box::new(callback),
            ))),
            None,
        )
    })
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

fn typescript_ls() -> Package {
    let args = pkg_info!(
        "https://github.com/typescript-language-server/typescript-language-server",
        "typescript-language-server"
    );
    let callback = Box::new(|_op: CallbackOperation, _info: &PkgInfo, _dirs: &Dirs| Ok(()));
    let installer = Box::new(NpmInstaller::new(vec!["typescript".to_string()], callback));
    Package::new(args, None, Some(installer))
}

fn pyright() -> Package {
    let args = pkg_info!("https://github.com/microsoft/pyright", "pyright");
    let callback = Box::new(|_op: CallbackOperation, _info: &PkgInfo, _dirs: &Dirs| Ok(()));
    let installer = Box::new(NpmInstaller::new(vec![], callback));
    Package::new(args, None, Some(installer))
}

fn bash_ls() -> Package {
    let args = pkg_info!(
        "https://github.com/bash-lsp/bash-language-server",
        "bash-language-server"
    );
    let callback = Box::new(|_op: CallbackOperation, _info: &PkgInfo, _dirs: &Dirs| Ok(()));
    let installer = Box::new(NpmInstaller::new(vec![], callback));
    Package::new(args, None, Some(installer))
}

pub fn pylsp() -> Package {
    let args = pkg_info!(
        "https://github.com/python-lsp/python-lsp-server",
        "pylsp",
        "python-lsp-server",
        "pylsp"
    );
    let installer = Box::new(PipInstaller::new(vec![]));
    Package::new(args, None, Some(installer))
}
