use imk::core::config::InputConfig;

#[test]
fn parse_config_eng_and_chinese() {
    let json = r#"{"ENG":["notepad.exe"],"CHINESE":["wechat.exe"]}"#;
    let cfg = InputConfig::from_json_str(json).unwrap();
    assert!(cfg.eng.contains("notepad.exe"));
    assert!(cfg.chinese.contains("wechat.exe"));
}
