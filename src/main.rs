use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use gpui::Application;

use imk::core::config::InputConfig;
use imk::core::mode_manager::{InputMode as CoreInputMode, ModeDecision};
use imk::platform_win::ime::{ImeController, InputMode as ImeInputMode};
use imk::platform_win::winapi::get_process_name;
use imk::service::window_monitor::WindowMonitor;
use imk::tray::tray::Tray;
use imk::ui::settings::SettingsWindow;

use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--settings") {
        run_settings_window();
        return;
    }

    run_tray_app();
}

fn run_settings_window() {
    Application::new().run(|cx| {
        let _ = SettingsWindow::open(cx);
        cx.activate(true);
    });
}

fn run_tray_app() {
    let config_path = default_config_path();
    let ime = Arc::new(ImeController::new());
    let mut monitor = WindowMonitor::new();

    {
        let ime = ime.clone();
        let config_path = config_path.clone();
        monitor.set_callback(move |hwnd| {
            let exe = match get_process_name(hwnd) {
                Some(exe) => exe,
                None => return,
            };

            let cfg = InputConfig::from_file(&config_path).unwrap_or_default();
            let is_english = ime.is_english().unwrap_or(true);

            let eng_list: Vec<&str> = cfg.eng.iter().map(String::as_str).collect();
            let chn_list: Vec<&str> = cfg.chinese.iter().map(String::as_str).collect();

            let decision = ModeDecision::for_process(&exe, is_english, &eng_list, &chn_list);
            match decision {
                Some(CoreInputMode::English) => {
                    let _ = ime.set_mode(ImeInputMode::English);
                }
                Some(CoreInputMode::Chinese) => {
                    let _ = ime.set_mode(ImeInputMode::Chinese);
                }
                None => {}
            }
        });
    }

    monitor.start();

    let mut tray = Tray::new();
    tray.set_callbacks(
        move || {
            if let Ok(exe) = std::env::current_exe() {
                let _ = Command::new(exe).arg("--settings").spawn();
            }
        },
        move || unsafe {
            PostQuitMessage(0);
        },
    );

    tray.run_message_loop();
    monitor.stop();
}

fn default_config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("IMK").join("default_input_config.json")
}
