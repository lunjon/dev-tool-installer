use crate::config::Config;
use crate::github::GitHubClient;
use crate::pkg::{Dirs, Entry, Manifest};
use crate::pkg::{Package, Version};
use crate::util;
use crate::{cli::*, pkgs};
use anyhow::{bail, Result};
use crossterm::style::Stylize;
use prettytable::{format, row, Table};
use std::thread;
use std::{fs, path::PathBuf};

#[cfg(test)]
mod tests;

pub struct Handler {
    config_filepath: PathBuf,
    bin_dir: PathBuf,
    pkg_dir: PathBuf,
    dirs: Dirs,
    manifest_path: PathBuf,
}

struct Context {
    manifest: Manifest,
    config: Config,
    packages: pkgs::Packages,
    gh: GitHubClient,
}

impl Handler {
    pub fn new(root: PathBuf) -> Self {
        let cfg = root.join("config.toml");
        let bin_dir = root.join("bin");
        let pkg_dir = root.join("pkg");
        let manifest_path = root.join("manifest.json");

        let dirs = Dirs {
            root_dir: root,
            bin_dir: bin_dir.clone(),
            pkg_dir: pkg_dir.clone(),
        };

        Self {
            config_filepath: cfg,
            bin_dir,
            pkg_dir,
            dirs,
            manifest_path,
        }
    }

    pub fn handle(&self, cli: Cli) -> Result<()> {
        let mut cx = self.bootstrap()?;

        match cli.command {
            Command::Info => self.handle_info(&cx)?,
            Command::Check(args) => self.handle_check(&cx, args)?,
            Command::List(args) => self.handle_list(&cx, args)?,
            Command::Install(args) => self.handle_install(&mut cx, args.name, args.version)?,
            Command::Remove { name } => self.handle_uninstall(&mut cx, name)?,
            Command::Update(args) => self.handle_update(&mut cx, args.name, args.version)?,
        };

        self.write_manifest(&cx.manifest)
    }

    fn handle_info(&self, cx: &Context) -> Result<()> {
        println!("Directories:");
        println!("  configuration:  {}", self.config_filepath.display());
        println!("  binaries:       {}", self.bin_dir.display());
        println!("  packages:       {}", self.pkg_dir.display());
        println!();

        print_platform();
        println!();

        let count = cx.manifest.installed_count();
        let suffix = if count == 1 { "" } else { "s" };
        println!("{} installed package{}.", count, suffix);

        Ok(())
    }

    fn handle_check(&self, cx: &Context, _args: CheckArgs) -> Result<()> {
        let mut results: Vec<(bool, String)> = Vec::new();

        // Check for versions i parallel using a thread scope.
        thread::scope(|s| {
            let mut handles: Vec<thread::ScopedJoinHandle<(bool, String)>> = Vec::new();

            for entry in &cx.manifest.packages {
                let pkg = match cx.packages.get(&entry.name) {
                    Some(pkg) => pkg,
                    None => continue,
                };

                let h = s.spawn(|| match cx.gh.latest(pkg.repo()) {
                    Ok(release) => match release {
                        Some(release) => {
                            let version = entry.version.to_string();
                            if release.tag != version {
                                let icon = "".yellow();
                                let output = format!(
                                    "{} {}: {}  {}",
                                    icon,
                                    entry.name.as_str().bold(),
                                    version,
                                    release.tag,
                                );

                                (false, output)
                            } else {
                                let icon = "".green();
                                let output =
                                    format!("{} {}: {}", icon, entry.name.as_str().bold(), version);
                                (true, output)
                            }
                        }
                        None => {
                            let icon = "?".yellow();
                            (
                                false,
                                format!("{} {}: unable resolve version", icon, pkg.name()),
                            )
                        }
                    },
                    Err(err) => (
                        false,
                        format!("error when resolving release for {}: {}", pkg.name(), err),
                    ),
                });

                handles.push(h);
            }

            for handle in handles {
                if let Ok(r) = handle.join() {
                    results.push(r);
                }
            }
        });

        results.sort_by_key(|res| res.0);
        for (_, res) in results {
            println!("{}", res);
        }

        Ok(())
    }

    fn handle_list(&self, cx: &Context, args: ListArgs) -> Result<()> {
        let mut pkgs: Vec<(bool, String, Option<Version>)> = cx
            .packages
            .keys()
            .map(|k| match cx.manifest.get(k) {
                Some(pkg) => (true, pkg.name.to_string(), Some(pkg.version.clone())),
                None => (false, k.to_string(), None),
            })
            .filter(|(installed, _, _)| *installed || args.all)
            .collect();

        // Sort so uninstalled are last in the list.
        pkgs.sort_by_key(|pkg| !pkg.0);

        if args.detailed {
            self.detailed_list_packages(cx, pkgs);
        } else {
            self.simple_list_packages(pkgs)
        }

        Ok(())
    }

    fn simple_list_packages(&self, pkgs: Vec<(bool, String, Option<Version>)>) {
        for (installed, name, version) in pkgs {
            if installed {
                let version = version.unwrap().to_string();
                println!("{}: {}", name.to_string().bold(), version);
            } else {
                println!("{}", name.to_string().dark_grey());
            }
        }
    }

    fn detailed_list_packages(&self, cx: &Context, pkgs: Vec<(bool, String, Option<Version>)>) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        table.add_row(row![
            "Package".to_string().green().bold(),
            "Version".to_string().green().bold(),
            "Repository".to_string().green().bold(),
        ]);

        for (_, name, version) in pkgs {
            let pkg = cx.packages.get(&name).unwrap();
            let row = if let Some(version) = version {
                row![name, version.to_string(), pkg.repo()]
            } else {
                row![name.dark_grey(), "", pkg.repo()]
            };

            table.add_row(row);
        }

        table.printstd();
    }

    fn handle_install(
        &self,
        cx: &mut Context,
        name: Option<String>,
        version: Option<String>,
    ) -> Result<()> {
        self.ensure_install(cx)?;

        if let Some(name) = name {
            let pkg = match cx.packages.get(&name) {
                Some(pkg) => pkg,
                None => bail!("unknown package: {}", name),
            };

            if !cx.manifest.installed(&name) {
                println!("Installing {}...", pkg.name().as_str().green());
                let version = self.install_pkg(&cx.gh, &mut cx.manifest, pkg, version)?;

                match version {
                    Version::Unknown(v) => println!("Installed unknown version {}.", v),
                    v => println!("Installed version {}.", v),
                }
            } else {
                eprintln!(
                    "{} already installed. Use 'update' to update.",
                    name.as_str().green()
                );
            }
        }

        Ok(())
    }

    fn install_pkg(
        &self,
        gh: &GitHubClient,
        manifest: &mut Manifest,
        pkg: &Package,
        vrs: Option<String>,
    ) -> Result<Version> {
        let vrs = match vrs {
            Some(v) => Some(Version::try_from(&v)?),
            None => None,
        };

        let release = gh.try_get_release(pkg.repo(), vrs)?;
        let version = pkg.install(release, &self.dirs)?;

        let entry = Entry::new(pkg.name().to_string(), version.clone());
        manifest.upsert(entry);
        Ok(version)
    }

    fn ensure_install(&self, cx: &mut Context) -> Result<()> {
        if let Some(pkgs) = &cx.config.ensure_installed {
            for name in pkgs {
                let pkg = match cx.packages.get(name) {
                    Some(pkg) => pkg,
                    None => bail!("unknown package from ensure installed: {}", name),
                };

                if !cx.manifest.installed(name) {
                    log::info!(
                        "Found package from ensure-installed that wasn't installed: {}",
                        name
                    );

                    print!("Installing {}... ", name);
                    let version = self.install_pkg(&cx.gh, &mut cx.manifest, pkg, None)?;
                    println!("Installed version {}", version);
                }
            }
        }
        Ok(())
    }

    fn handle_uninstall(&self, cx: &mut Context, name: String) -> Result<()> {
        let pkg = match cx.packages.get(&name) {
            Some(pkg) => pkg,
            None => bail!("unknown package: {}", name),
        };

        if !cx.manifest.installed(&name) {
            eprintln!("{} not installed", name);
            return Ok(());
        }

        println!("Uninstalling {}... ", &name);
        pkg.uninstall(&self.dirs)?;

        cx.manifest.remove(&name);
        Ok(())
    }

    fn handle_update(&self, cx: &mut Context, name: String, version: Option<String>) -> Result<()> {
        let pkg = match cx.packages.get(&name) {
            Some(pkg) => pkg,
            None => bail!("unknown package: {}", name),
        };

        if !cx.manifest.installed(&name) {
            eprintln!("{} not installed", name);
            return Ok(());
        }

        let version = match version {
            Some(v) => Some(Version::try_from(&v)?),
            None => None,
        };

        let release = cx.gh.try_get_release(pkg.repo(), version)?;

        let version = pkg.update(release, &self.dirs)?;
        let entry = Entry::new(pkg.name().to_string(), version.clone());
        cx.manifest.upsert(entry);

        println!("Updated {} to version {}", name, version);
        Ok(())
    }

    fn bootstrap(&self) -> Result<Context> {
        if !self.bin_dir.exists() {
            fs::create_dir_all(&self.bin_dir)?;
        }

        let manifest = if !self.manifest_path.exists() {
            log::info!("No manifest file found - creating new");
            let manifest = Manifest::default();
            self.write_manifest(&manifest)?;
            manifest
        } else {
            log::debug!("Loading manifest from {:?}", self.manifest_path);
            let manifest: Manifest = util::json_from_file(&self.manifest_path)?;
            manifest
        };

        let config = Config::load_or_default(&self.config_filepath)?;
        let packages = pkgs::get_packages(&config)?;

        let gh = GitHubClient::new(&config);

        Ok(Context {
            gh,
            manifest,
            config,
            packages,
        })
    }

    fn write_manifest(&self, manifest: &Manifest) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.manifest_path)?;
        serde_json::to_writer_pretty(&mut file, manifest)?;

        log::info!("Successfully wrote manifest at {:?}", self.manifest_path);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn print_platform() {
    println!("Platform: windows");
}

#[cfg(target_os = "linux")]
fn print_platform() {
    println!("Platform: linux");
}

#[cfg(target_os = "macos")]
fn print_platform() {
    println!("Platform: macos");
}
