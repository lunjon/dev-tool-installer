use super::gh_client;
use crate::config::Config;
use crate::pkg::{CargoInstaller, Dirs, GithubReleaseInstaller, GoInstaller, Package, PkgInfo};
use crate::{pkg_info, util};
use anyhow::bail;
use std::fs;
use std::path::Path;

pub fn packages(cfg: &Config) -> Vec<Package> {
    let mut packages = vec![
        nushell(cfg),
        bat(cfg),
        fd(cfg),
        just(cfg),
        exa(cfg),
        lazygit(),
        goimports(),
    ];

    let maybe_packages = vec![direnv(cfg), broot(cfg)];
    packages.extend(maybe_packages.into_iter().flatten());

    packages
}

fn nushell(cfg: &Config) -> Package {
    let repo = "https://github.com/nushell/nushell";
    let info = pkg_info!(&repo, "nushell", "nu", "nu");

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
        "nu-.*-x86_64-linux-musl-full.tar.gz"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        "nu-.*-x86_64-linux-gnu-full.tar.gz"
    } else {
        return Package::new(info, None, Some(Box::new(CargoInstaller {})));
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            asset_regex.to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        Some(Box::new(CargoInstaller {})),
    )
}

fn bat(cfg: &Config) -> Package {
    let repo = "https://github.com/sharkdp/bat";
    let info = pkg_info!(&repo, "bat");

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
        "bat-.*-x86_64-unknown-linux-musl.tar.gz"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        "bat-.*-x86_64-unknown-linux-gnu.tar.gz"
    } else {
        return Package::new(info, None, Some(Box::new(CargoInstaller {})));
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            asset_regex.to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        Some(Box::new(CargoInstaller {})),
    )
}

fn just(cfg: &Config) -> Package {
    let repo = "https://github.com/casey/just";
    let info = pkg_info!(&repo, "just");

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
        "just-.*-x86_64-unknown-linux-musl.tar.gz"
    } else {
        return Package::new(info, None, Some(Box::new(CargoInstaller {})));
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            asset_regex.to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        Some(Box::new(CargoInstaller {})),
    )
}

fn exa(cfg: &Config) -> Package {
    let repo = "https://github.com/ogham/exa";
    let info = pkg_info!(&repo, "exa");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let pkg_dir = dirs.pkg_dir.join(&info.name);
        util::decompress(path, &pkg_dir)?;

        let pkg_bin = pkg_dir.join("bin").join(&info.name);
        let bin = dirs.bin_dir.join(&info.name);
        fs::rename(pkg_bin, &bin)?;
        util::make_executable(&bin)?;

        fs::remove_dir_all(&pkg_dir)?;
        Ok(())
    };

    let asset_regex = if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "musl"
    )) {
        "exa-linux-x86_64-musl-.*.zip"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "exa-linux-x86_64-.*.zip"
    } else {
        return Package::new(info, None, Some(Box::new(CargoInstaller {})));
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            asset_regex.to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        Some(Box::new(CargoInstaller {})),
    )
}

fn fd(cfg: &Config) -> Package {
    let repo = "https://github.com/sharkdp/fd";
    let info = pkg_info!(&repo, "fd", "fd-find");

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
        "fd-.*-x86_64-unknown-linux-musl.tar.gz"
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
        "fd-.*-x86_64-unknown-linux-gnu.tar.gz"
    } else {
        return Package::new(info, None, Some(Box::new(CargoInstaller {})));
    };

    Package::new(
        info,
        Some(Box::new(GithubReleaseInstaller::new(
            asset_regex.to_string(),
            gh_client(cfg),
            Box::new(callback),
        ))),
        Some(Box::new(CargoInstaller {})),
    )
}

fn direnv(cfg: &Config) -> Option<Package> {
    let repo = "https://github.com/direnv/direnv";
    let args = pkg_info!(&repo, "direnv");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let bin = dirs.bin_dir.join(&info.name);
        fs::rename(path, &bin)?;
        util::make_executable(&bin)?;

        Ok(())
    };

    let asset_regex = if cfg!(all(target_os = "linux", target_arch = "x86_64",)) {
        Some("direnv.linux-amd64")
    } else if cfg!(all(target_os = "linux", target_arch = "arm",)) {
        Some("direnv.linux-arm64")
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

fn broot(cfg: &Config) -> Option<Package> {
    let repo = "https://github.com/Canop/broot";
    let args = pkg_info!(&repo, "broot");

    let callback = |info: &PkgInfo, dirs: &Dirs, path: &Path| {
        let name = &info.name;
        let pkg_dir = dirs.pkg_dir.join(name);
        util::decompress(path, &pkg_dir)?;

        let pkg_bin = pkg_dir.join(name);
        let bin = dirs.bin_dir.join(name);
        fs::rename(pkg_bin, &bin)?;
        util::make_executable(&bin)?;

        fs::remove_dir_all(&pkg_dir)?;
        Ok(())
    };

    let asset_regex = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("broot-x86_64-unknown-linux-musl-.*.zip")
    } else if cfg!(all(target_os = "linux", target_arch = "arm",)) {
        Some("broot-aarch64-unknown-linux-musl-.*.zip")
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

fn lazygit() -> Package {
    let args = pkg_info!(
        "https://github.com/jesseduffield/lazygit",
        "lazygit",
        "github.com/jesseduffield/lazygit",
        "lazygit"
    );
    let installer = Box::new(GoInstaller {});
    Package::new(args, None, Some(installer))
}

fn goimports() -> Package {
    let args = pkg_info!(
        "https://github.com/golang/tools",
        "goimports",
        "golang.org/x/tools/cmd/goimports",
        "goimports"
    );
    let installer = Box::new(GoInstaller {});
    Package::new(args, None, Some(installer))
}
