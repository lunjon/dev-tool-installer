use std::fs;

use super::{gh_client, JsonPackage, Pkg};
use crate::{
    config::Config,
    pkg::{CallbackOperation, Dirs, Package, PkgInfo, NPM},
    pkg_args, util,
};
use anyhow::Result;

pub fn build(cfg: &Config, pkg: &JsonPackage) -> Result<Package> {
    let pkginfo: Pkg = serde_json::from_value(pkg.pkg.clone())?;
    let bin = match pkginfo.bin {
        Some(b) => b,
        None => pkg.name.clone(),
    };

    let args = pkg_args!(pkg.name, pkginfo.module, bin);
    let gh = gh_client(cfg, &pkg.repo);

    let name = pkg.name.clone();
    let callback =
        Box::new(
            move |op: CallbackOperation, info: &PkgInfo, dirs: &Dirs| match name.as_str() {
                "vscode-langservers-extracted" => {
                    vscode_langservers_extracted_callback(op, info, dirs)
                }
                _ => Ok(()),
            },
        );
    let symlink = pkg.name != "vscode-langservers-extracted";

    let installer = Box::new(NPM::new(symlink, pkginfo.deps, callback));
    let p = Package::new(args, installer, gh);
    Ok(p)
}

fn vscode_langservers_extracted_callback(
    op: CallbackOperation,
    info: &PkgInfo,
    dirs: &Dirs,
) -> Result<()> {
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
                util::link(&pkg_bin, &bin)?;
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
}
