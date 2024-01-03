use clap::{Args, Parser, Subcommand};

/// dti -- manage your code tools with ease.
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Display info about the system.
    Info,
    /// Checks for updates of packages.
    Check(CheckArgs),
    /// List installed and available packages.
    #[command(visible_alias = "ls")]
    List(ListArgs),
    /// Install a package.
    /// Packages from ensure-installed will also be installed.
    #[command(visible_alias = "i")]
    Install {
        name: Option<String>,
        /// Use as version to install. Else try to resolve latest version.
        version: Option<String>,
    },
    /// Uninstall a package.
    #[command(visible_alias = "remove", visible_alias = "rm")]
    Uninstall { name: String },
    /// Updates a package.
    #[command(visible_alias = "up")]
    Update {
        name: String,
        /// Version to update, or downgrade, to.
        /// Upgrades to latest by default.
        version: Option<String>,
    },
}

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub all: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// Lists all packages.
    #[arg(long, short)]
    pub all: bool,
}
