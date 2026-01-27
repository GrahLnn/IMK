use imk::platform_win::ime::{ImeController, InputMode};

#[test]
fn ime_controller_compiles() {
    let _ = ImeController::new();
    let _ = InputMode::English;
}
