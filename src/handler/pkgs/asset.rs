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
        "omnisharp" => omnisharp_callback(info, dirs, path),
        "direnv" => direnv_callback(info, dirs, path),
        s => bail!("unknown package: {}", s),
    };

    let p = Package::new(
        pkg_args!(pkg.name, pkg.name, bin),
        Box::new(GithubRelease::new(
            &pkginfo.asset_regex,
            gh_client(cfg, &pkg.repo),
            Box::new(callback),
        )),
        gh_client(cfg, &pkg.repo),
    );
    Ok(p)
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
    util::create_script(&bin, lines, &opt)?;

    Ok(())
}

fn elixirls_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let pkg_dir = dirs.pkg_dir.join(&info.name);

    util::decompress(path, &pkg_dir)?;

    let executable = pkg_dir.join("language_server.sh");
    let bin = dirs.bin_dir.join(&info.name);
    util::link(&executable, &bin)?;

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
    util::link(&pkg_bin, &bin)
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
            util::link(&pkg_bin, &bin)?;
            return Ok(());
        }
    }

    bail!("failed to locate installed package")
}

fn omnisharp_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let pkg_dir = dirs.pkg_dir.join(&info.name);
    util::decompress(path, &pkg_dir)?;

    let executable = pkg_dir.join("OmniSharp");
    let bin = dirs.bin_dir.join("OmniSharp");
    util::link(&executable, &bin)?;

    Ok(())
}

fn direnv_callback(info: &PkgInfo, dirs: &Dirs, path: &Path) -> Result<()> {
    let executable = path;
    util::make_executable(executable)?;

    let bin = dirs.bin_dir.join(&info.name);
    util::link(executable, &bin)?;

    Ok(())
}
