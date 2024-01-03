use crate::config::Config;

use super::pkgs::get_packages;

#[test]
fn test_get_packages() {
    let cfg = Config::default();
    let res = get_packages(&cfg);
    assert!(res.is_ok());
}
