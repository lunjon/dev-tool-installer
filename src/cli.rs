use clap::{Args, Parser, Subcommand};

/// dti -- manage your code tools with ease.
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
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
    #[command(visible_alias = "i")]
    Install(InstallArgs),
    /// Uninstall a package.
    #[command(visible_alias = "remove", visible_alias = "rm")]
    Uninstall { name: String },
    /// Updates a package.
    #[command(visible_alias = "up")]
    Update(UpdateArgs),
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
    /// Output detailed information in a table format.
    #[arg(long, short)]
    pub detailed: bool,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Name of the package to install or update.
    /// See available packages with "list --all".
    #[arg()]
    pub name: Option<String>,
    /// Install specified version.
    /// Latest version is resolved by default.
    #[arg(long, short)]
    pub version: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Name of the package to update.
    /// See installed packages with "check".
    #[arg()]
    pub name: String,
    /// Install specified version.
    /// Latest version is resolved by default.
    #[arg(long, short)]
    pub version: Option<String>,
}
