#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use gpui::Application;

use imk::core::config::{load_config_or_default, InputConfig};
use imk::core::mode_manager::{InputMode as CoreInputMode, ModeDecision};
use imk::platform_win::ime::{ImeController, InputMode as ImeInputMode};
use imk::platform_win::winapi::get_process_name;
use imk::service::caps_interceptor::CapsInterceptor;
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
    let caps_interceptor = Arc::new(Mutex::new(CapsInterceptor::new()));
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

    if load_config_or_default(&config_path).caps_interceptor {
        if let Ok(mut caps) = caps_interceptor.lock() {
            caps.start();
        }
    }

    let mut tray = Tray::new();
    tray.set_callbacks(
        move || {
            if let Ok(exe) = std::env::current_exe() {
                let _ = Command::new(exe).arg("--settings").spawn();
            }
        },
        {
            let caps_interceptor = caps_interceptor.clone();
            move || unsafe {
                if let Ok(mut caps) = caps_interceptor.lock() {
                    caps.stop();
                }
                PostQuitMessage(0);
            }
        },
        {
            let caps_interceptor = caps_interceptor.clone();
            move |enabled| {
                if let Ok(mut caps) = caps_interceptor.lock() {
                    if enabled {
                        caps.start();
                    } else {
                        caps.stop();
                    }
                }
            }
        },
    );

    tray.run_message_loop();
    monitor.stop();
}

fn default_config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_default();
    base.join("IMK").join("default_input_config.json")
}
