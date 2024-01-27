use super::gh_client;
use crate::config::Config;
use crate::pkg::{Dirs, GithubRelease, Package, PkgInfo};
use crate::{pkg_args, util};
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn packages(cfg: &Config) -> Result<Vec<Package>> {
    let packages = vec![
        elixir_ls(cfg)?,
        rust_analyzer(cfg)?,
        clojure_lsp(cfg)?,
        ltex_ls(cfg)?,
        direnv(cfg)?,
    ];
    Ok(packages)
}

fn elixir_ls(cfg: &Config) -> Result<Package> {
    let repo = "https://github.com/elixir-lsp/elixir-ls";
    let args = pkg_args!(&repo, "elixir-ls");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);

        util::decompress(path, &pkg_dir)?;

        let executable = pkg_dir.join("language_server.sh");
        let bin = dirs.bin_dir.join(&info.name);
        util::symlink(&executable, &bin)?;

        Ok(())
    };

    let asset_regex = get_asset_regex(&args.name)?;

    Ok(Package::new(
        args,
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg),
            Box::new(callback),
        )),
    ))
}

fn rust_analyzer(cfg: &Config) -> Result<Package> {
    let repo = "https://github.com/rust-lang/rust-analyzer";
    let args = pkg_args!(&repo, "rust-analyzer");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let bin = dirs.bin_dir.join(&info.name);
        util::decompress(path, &bin)?;
        util::make_executable(&bin)?;
        Ok(())
    };

    let asset_regex = get_asset_regex(&args.name)?;

    Ok(Package::new(
        args,
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg),
            Box::new(callback),
        )),
    ))
}

fn clojure_lsp(cfg: &Config) -> Result<Package> {
    let repo = "https://github.com/clojure-lsp/clojure-lsp";
    let args = pkg_args!(&repo, "clojure-lsp");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        let pkg_bin = pkg_dir.join(&info.bin_name);
        let bin = dirs.bin_dir.join(&info.bin_name);
        util::symlink(&pkg_bin, &bin)
    };

    let asset_regex = get_asset_regex(&args.name)?;

    Ok(Package::new(
        args,
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg),
            Box::new(callback),
        )),
    ))
}

fn ltex_ls(cfg: &Config) -> Result<Package> {
    let repo = "https://github.com/valentjn/ltex-ls";
    let args = pkg_args!(&repo, "ltex-ls");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        // The archive contains a directory with the installed version
        // in its name, which we do not know here. So try to find
        // the directory.
        for entry in (fs::read_dir(&pkg_dir)?).flatten() {
            let path = entry.path();
            if path.is_dir() {
                let pkg_bin = pkg_dir.join(&path).join("bin").join(&info.bin_name);
                let bin = dirs.bin_dir.join(&info.bin_name);
                util::symlink(&pkg_bin, &bin)?;
                return Ok(());
            }
        }

        bail!("failed to locate installed package")
    };

    let asset_regex = get_asset_regex(&args.name)?;

    Ok(Package::new(
        args,
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg),
            Box::new(callback),
        )),
    ))
}

fn direnv(cfg: &Config) -> Result<Package> {
    let repo = "https://github.com/direnv/direnv";
    let args = pkg_args!(&repo, "direnv");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let executable = path;
        util::make_executable(executable)?;

        let bin = dirs.bin_dir.join(&info.name);
        util::symlink(executable, &bin)?;

        Ok(())
    };

    let asset_regex = get_asset_regex(&args.name)?;

    Ok(Package::new(
        args,
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg),
            Box::new(callback),
        )),
    ))
}

/// Gets a regex for the asset name (from the GitHub release)
/// based on the platform (OS + Arch).
fn get_asset_regex(name: &str) -> Result<String> {
    let re = match name {
        "rust-analyzer" => {
            // Linux
            if cfg!(all(
                target_os = "linux",
                target_arch = "x86_64",
                target_env = "musl"
            )) {
                "rust-analyzer-x86_64-unknown-linux-musl.gz"
            } else if cfg!(all(
                target_os = "linux",
                target_arch = "x86_64",
                target_env = "gnu"
            )) {
                "rust-analyzer-x86_64-unknown-linux-gnu.gz"
            } else {
                bail!("unable to determine platform for package: {}", name)
            }
        }
        "elixir-ls" => "^elixir-ls-v.*\\.zip$",
        "clojure-lsp" => {
            // Linux
            if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                "clojure-lsp-native-linux-amd64.zip"
            } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
                "clojure-lsp-native-linux-aarch64.zip"
            } else {
                bail!("unable to determine platform for package: {}", name)
            }
        }
        "ltex-ls" => {
            if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                "ltex-ls-.*-linux-x64.tar.gz"
            } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
                "ltex-ls-.*-windows-x64.zip"
            } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
                "ltex-ls-.*-mac-x64.zip"
            } else {
                bail!("unable to determine platform for package: {}", name)
            }
        }
        "direnv" => {
            // Linux
            if cfg!(all(target_os = "linux", target_arch = "x86_64",)) {
                "direnv.linux-amd64"
            } else if cfg!(all(target_os = "linux", target_arch = "arm",)) {
                "direnv.linux-arm64"
            } else {
                bail!("unable to determine platform for package: {}", name)
            }
        }
        s => bail!("unknown package: {}", s),
    };

    Ok(re.to_string())
}
