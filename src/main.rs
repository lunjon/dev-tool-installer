use clap::Parser;
use crossterm::style::Stylize;
use devtoolinstaller::cli::Cli;
use devtoolinstaller::handler::Handler;
use std::path::PathBuf;

fn main() {
    let basedirs = match directories::BaseDirs::new() {
        Some(dirs) => dirs,
        None => {
            eprintln!("Failed to resolve user directories");
            std::process::exit(1);
        }
    };

    let dir = match std::env::var("DTI_ROOT") {
        Ok(dir) => PathBuf::from(&dir),
        Err(_) => basedirs.home_dir().join(".devtoolinstaller"),
    };

    let handler = Handler::new(dir);
    let cli = Cli::parse();
    if let Err(err) = handler.handle(cli) {
        eprintln!("{}: {}", "error".red(), err);
    }
}
