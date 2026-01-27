use imk::core::mode_manager::{InputMode, ModeDecision};

#[test]
fn decide_switch_to_eng_when_in_list() {
    let decision = ModeDecision::for_process("notepad.exe", false, &["notepad.exe"], &[]);
    assert_eq!(decision, Some(InputMode::English));
}
