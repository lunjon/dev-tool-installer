use super::gh_client;
use crate::config::Config;
use crate::pkg::{Cargo, Dirs, GithubRelease, Package, PkgInfo};
use crate::{pkg_info, util};
use anyhow::bail;
use std::fs;
use std::path::Path;

pub fn packages(cfg: &Config) -> Vec<Package> {
    let packages = vec![
        elixir_ls(cfg),
        rust_analyzer(cfg),
        bat(cfg),
        just(cfg),
        clojure_lsp(cfg),
        direnv(cfg),
    ];

    packages.into_iter().filter_map(|opt| opt).collect()
}

fn elixir_ls(cfg: &Config) -> Option<Package> {
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

    Some(Package::new(
        info,
        Some(Box::new(GithubRelease::new(
            "^elixir-ls-v.*\\.zip$",
            gh_client(cfg),
            Box::new(callback),
        ))),
        None,
    ))
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
            Some(Box::new(GithubRelease::new(
                pattern,
                gh_client(cfg),
                Box::new(callback),
            ))),
            None,
        )
    })
}

fn bat(cfg: &Config) -> Option<Package> {
    let repo = "sharkdp/bat";
    let args = pkg_info!(&repo, "bat");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        let dirname = path
            .file_name()
            .map(|dir| dir.to_str().unwrap().trim_end_matches(".tar.gz"));

        match dirname {
            Some(dirname) => {
                let pkg_bin = pkg_dir.join(dirname).join(&info.bin_name);
                let bin = dirs.bin_dir.join(&info.bin_name);
                fs::rename(pkg_bin, &bin)?;
                util::make_executable(&bin)?;

                fs::remove_dir_all(pkg_dir)?;

                Ok(())
            }
            None => bail!("failed to install release artifact for {}", info.name),
        }
    };

    let asset_regex = if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "musl"
    )) {
        Some("bat-.*-x86_64-unknown-linux-musl.tar.gz")
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        Some("bat-.*-x86_64-unknown-linux-gnu.tar.gz")
    } else {
        None
    };

    asset_regex.map(|pattern| {
        Package::new(
            args,
            Some(Box::new(GithubRelease::new(
                pattern,
                gh_client(cfg),
                Box::new(callback),
            ))),
            Some(Box::new(Cargo {})),
        )
    })
}

fn just(cfg: &Config) -> Option<Package> {
    let repo = "casey/just";
    let args = pkg_info!(&repo, "just");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        let pkg_bin = pkg_dir.join(&info.name);
        let bin = dirs.bin_dir.join(&info.name);
        fs::rename(pkg_bin, &bin)?;
        util::make_executable(&bin)?;

        fs::remove_dir_all(&pkg_dir)?;
        Ok(())
    };

    let asset_regex = if cfg!(all(target_os = "linux", target_arch = "x86_64",)) {
        Some("just-.*-x86_64-unknown-linux-musl.tar.gz")
    } else {
        None
    };

    asset_regex.map(|pattern| {
        Package::new(
            args,
            Some(Box::new(GithubRelease::new(
                pattern,
                gh_client(cfg),
                Box::new(callback),
            ))),
            Some(Box::new(Cargo {})),
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
            Some(Box::new(GithubRelease::new(
                pattern,
                gh_client(cfg),
                Box::new(callback),
            ))),
            None,
        )
    })
}

fn direnv(cfg: &Config) -> Option<Package> {
    let repo = "https://github.com/direnv/direnv";
    let args = pkg_info!(&repo, "direnv");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let executable = path;
        util::make_executable(executable)?;

        let bin = dirs.bin_dir.join(&info.name);
        util::symlink(executable, &bin)?;

        Ok(())
    };

    let asset_regex = if cfg!(all(target_os = "linux", target_arch = "x86_64",)) {
        Some("direnv.linux-amd64")
    } else if cfg!(all(target_os = "linux", target_arch = "arm",)) {
        Some("direnv.linux-arm64")
    } else {
        None
        // bail!("unable to determine platform for package: {}", name)
    };

    asset_regex.map(|pattern| {
        Package::new(
            args,
            Some(Box::new(GithubRelease::new(
                pattern,
                gh_client(cfg),
                Box::new(callback),
            ))),
            None,
        )
    })
}
