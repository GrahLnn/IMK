use std::fs;

use imk::core::config::{load_config_or_default, save_config, update_caps_interceptor};

#[test]
fn update_caps_interceptor_persists() {
    let dir = std::env::temp_dir().join(format!("imk_test_caps_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("cfg.json");

    let cfg = load_config_or_default(&path);
    assert!(cfg.caps_interceptor);

    let mut cfg2 = cfg.clone();
    cfg2.caps_interceptor = false;
    save_config(&path, &cfg2).unwrap();
    let cfg3 = load_config_or_default(&path);
    assert!(!cfg3.caps_interceptor);

    update_caps_interceptor(&path, true).unwrap();
    let cfg4 = load_config_or_default(&path);
    assert!(cfg4.caps_interceptor);

    let _ = fs::remove_file(&path);
}
