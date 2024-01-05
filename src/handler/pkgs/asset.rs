use super::{gh_client, AssetPkgInfo, JsonPackage};
use crate::config::Config;
use crate::pkg::{Dirs, GithubRelease, Package, PkgInfo};
use crate::{pkg_args, util};
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

pub fn build(cfg: &Config, pkg: &JsonPackage) -> Result<Package> {
    let pkginfo: AssetPkgInfo = serde_json::from_value(pkg.pkg.clone())?;
    let bin = match pkginfo.bin {
        Some(b) => b,
        None => pkg.name.clone(),
    };

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| match info.name.as_str() {
        "luals" => luals_callback(info, dirs, path),
        "elixir-ls" => elixirls_callback(info, dirs, path),
        "rust-analyzer" => rust_analyzer_callback(info, dirs, path),
        "clojure-lsp" => clojure_lsp_callback(info, dirs, path),
        "ltex-ls" => ltex_ls_callback(info, dirs, path),
        "direnv" => direnv_callback(info, dirs, path),
        s => bail!("unknown package: {}", s),
    };

    let asset_regex = get_asset_regex(&pkg.name)?;

    let p = Package::new(
        pkg_args!(pkg.repo, pkg.name, pkg.name, bin),
        Box::new(GithubRelease::new(
            &asset_regex,
            gh_client(cfg, &pkg.repo),
            Box::new(callback),
        )),
        gh_client(cfg, &pkg.repo),
    );
    Ok(p)
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

const LUALS: &str = "lua-language-server";

fn luals_callback(_info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let pkg_dir = dirs.pkg_dir.join(LUALS);
    let pkg_bin = pkg_dir.join("bin").join(LUALS);
    util::decompress(path, &pkg_dir)?;

    let line = format!(r#"exec "{}" "$@""#, pkg_bin.display());
    let lines = vec![line];

    let opt = util::ScriptOptions::new();
    let bin = dirs.bin_dir.join(LUALS);
    opt.create(&bin, lines)?;

    Ok(())
}

fn elixirls_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let pkg_dir = dirs.pkg_dir.join(&info.name);

    util::decompress(path, &pkg_dir)?;

    let executable = pkg_dir.join("language_server.sh");
    let bin = dirs.bin_dir.join(&info.name);
    util::symlink(&executable, &bin)?;

    Ok(())
}

fn rust_analyzer_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let bin = dirs.bin_dir.join(&info.name);
    util::decompress(path, &bin)?;
    util::make_executable(&bin)?;
    Ok(())
}

fn clojure_lsp_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let pkg_dir = dirs.pkg_dir.join(&info.name);
    util::decompress(path, &pkg_dir)?;

    let pkg_bin = pkg_dir.join(&info.bin_name);
    let bin = dirs.bin_dir.join(&info.bin_name);
    util::symlink(&pkg_bin, &bin)
}

fn ltex_ls_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
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
}

fn direnv_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let executable = path;
    util::make_executable(executable)?;

    let bin = dirs.bin_dir.join(&info.name);
    util::symlink(executable, &bin)?;

    Ok(())
}
