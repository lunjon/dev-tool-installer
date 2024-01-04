use super::{pkgs::get_packages, Handler};
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

#[test]
fn test_get_packages() {
    let cfg = Config::default();
    let res = get_packages(&cfg);
    assert!(res.is_ok());
}

#[test]
fn test_bootstrap() -> Result<()> {
    let tx = TestContext::new();
    let cx = tx.handler.bootstrap()?;
    assert!(cx.packages.contains_key("gopls"));
    Ok(())
}

#[test]
fn test_handle_info() -> Result<()> {
    let tx = TestContext::new();
    let cx = tx.handler.bootstrap()?;
    tx.handler.handle_info(&cx)?;
    Ok(())
}

#[test]
fn test_install_go_package() -> Result<()> {
    let tx = TestContext::new();
    let mut cx = tx.handler.bootstrap()?;
    tx.handler
        .handle_install(&mut cx, Some(String::from("lazygit")), None)?;
    Ok(())
}

#[test]
fn test_install_release_asset_package() -> Result<()> {
    let tx = TestContext::new();
    let mut cx = tx.handler.bootstrap()?;
    tx.handler
        .handle_install(&mut cx, Some(String::from("rust-analyzer")), None)?;
    Ok(())
}

struct TestContext {
    handler: Handler,
    _dir: tempfile::TempDir,
}

impl TestContext {
    fn new() -> Self {
        let dir = tempfile::tempdir_in(".").expect("failed to create tempdir");
        let handler = Handler::new(PathBuf::from(dir.path()));
        Self { handler, _dir: dir }
    }
}
