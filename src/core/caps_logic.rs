#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsAction {
    SwitchIme,
    SetCaps(bool),
}

const CAPS_TAP_THRESHOLD_MS: f64 = 300.0;

pub fn decide_caps_action(held_ms: f64) -> CapsAction {
    if held_ms < CAPS_TAP_THRESHOLD_MS {
        CapsAction::SwitchIme
    } else {
        CapsAction::SetCaps(true)
    }
}

pub fn should_turn_caps_off_on_shift(caps_on: bool) -> bool {
    caps_on
}
