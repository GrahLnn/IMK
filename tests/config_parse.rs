use imk::core::config::InputConfig;

#[test]
fn parse_config_eng_and_chinese() {
    let json = r#"{"ENG":["notepad.exe"],"CHINESE":["wechat.exe"]}"#;
    let cfg = InputConfig::from_json_str(json).unwrap();
    assert!(cfg.eng.contains("notepad.exe"));
    assert!(cfg.chinese.contains("wechat.exe"));
}

#[test]
fn parse_config_caps_interceptor_flag() {
    let json = r#"{"ENG":[],"CHINESE":[],"CAPS_INTERCEPTOR":false}"#;
    let cfg = InputConfig::from_json_str(json).unwrap();
    assert!(!cfg.caps_interceptor);
}

#[test]
fn parse_config_caps_interceptor_default_true() {
    let json = r#"{"ENG":[],"CHINESE":[]}"#;
    let cfg = InputConfig::from_json_str(json).unwrap();
    assert!(cfg.caps_interceptor);
}
