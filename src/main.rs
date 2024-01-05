use clap::Parser;
use crossterm::style::Stylize;
use devtoolinstaller::cli::Cli;
use devtoolinstaller::handler::Handler;
use log::LevelFilter;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();
    let level = match cli.verbose {
        1 => Some(LevelFilter::Info),
        2 => Some(LevelFilter::Debug),
        3 => Some(LevelFilter::Trace),
        _ => None,
    };

    if let Some(level) = level {
        env_logger::Builder::new()
            .filter(None, LevelFilter::Off)
            .filter_module("devtoolinstaller", level)
            .init();
    }

    let basedirs = match directories::BaseDirs::new() {
        Some(dirs) => dirs,
        None => {
            eprintln!("Failed to resolve user directories");
            std::process::exit(1);
        }
    };

    let dir = match std::env::var("DTI_ROOT") {
        Ok(dir) => {
            log::info!("Using root from environment variable: {}", dir);
            PathBuf::from(&dir)
        }
        Err(_) => basedirs.home_dir().join(".devtoolinstaller"),
    };

    let handler = Handler::new(dir);

    if let Err(err) = handler.handle(cli) {
        eprintln!("{}: {}", "error".red(), err);
        std::process::exit(1)
    }
}
