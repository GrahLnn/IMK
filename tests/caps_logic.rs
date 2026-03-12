use imk::core::caps_logic::{decide_caps_action, should_turn_caps_off_on_shift, CapsAction};

#[test]
fn short_press_switches_ime() {
    let action = decide_caps_action(200.0);
    assert_eq!(action, CapsAction::SwitchIme);
}

#[test]
fn long_press_turns_caps_on() {
    let action = decide_caps_action(400.0);
    assert_eq!(action, CapsAction::SetCaps(true));
}

#[test]
fn shift_turns_caps_off_when_caps_is_on() {
    assert!(should_turn_caps_off_on_shift(true));
}

#[test]
fn shift_does_not_toggle_when_caps_is_off() {
    assert!(!should_turn_caps_off_on_shift(false));
}
