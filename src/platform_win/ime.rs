use std::thread::sleep;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::{
    ImmGetContext, ImmGetConversionStatus, ImmGetDefaultIMEWnd, ImmReleaseContext,
    ImmSetConversionStatus, IME_CMODE_ALPHANUMERIC, IME_CMODE_NATIVE, IME_CONVERSION_MODE,
    IME_SENTENCE_MODE,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_IME_CONTROL};

const IMC_GETCONVERSIONMODE: usize = 0x0001;
const IMC_SETCONVERSIONMODE: usize = 0x0002;
const MAX_RETRY: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    English,
    Chinese,
}

pub struct ImeController;

impl ImeController {
    pub fn new() -> Self {
        Self
    }

    pub fn is_english(&self) -> Option<bool> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return None;
        }
        if let Some(val) = imm32_is_english(hwnd) {
            return Some(val);
        }
        black_magic_is_english(hwnd)
    }

    pub fn set_mode(&self, mode: InputMode) -> bool {
        for _ in 0..MAX_RETRY {
            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd.0.is_null() {
                sleep(Duration::from_millis(50));
                continue;
            }

            if imm32_set_mode(hwnd, mode) || black_magic_set_mode(hwnd, mode) {
                sleep(Duration::from_millis(30));
                if let Some(is_eng) = self.is_english() {
                    let ok = matches!((mode, is_eng), (InputMode::English, true) | (InputMode::Chinese, false));
                    if ok {
                        sleep(Duration::from_millis(100));
                        return true;
                    }
                }
            }
            sleep(Duration::from_millis(50));
        }
        false
    }
}

fn imm32_is_english(hwnd: HWND) -> Option<bool> {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return None;
        }
        let mut conversion = IME_CONVERSION_MODE(0);
        let mut sentence = IME_SENTENCE_MODE(0);
        let ok = ImmGetConversionStatus(himc, Some(&mut conversion), Some(&mut sentence)).as_bool();
        let _ = ImmReleaseContext(hwnd, himc);
        if !ok {
            return None;
        }
        let is_chinese = (conversion.0 & IME_CMODE_NATIVE.0) != 0;
        Some(!is_chinese)
    }
}

fn imm32_set_mode(hwnd: HWND, mode: InputMode) -> bool {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return false;
        }
        let mut conversion = IME_CONVERSION_MODE(0);
        let mut sentence = IME_SENTENCE_MODE(0);
        let ok = ImmGetConversionStatus(himc, Some(&mut conversion), Some(&mut sentence)).as_bool();
        if !ok {
            let _ = ImmReleaseContext(hwnd, himc);
            return false;
        }

        let new_conversion = match mode {
            InputMode::English => IME_CONVERSION_MODE(conversion.0 & !IME_CMODE_NATIVE.0),
            InputMode::Chinese => IME_CONVERSION_MODE(conversion.0 | IME_CMODE_NATIVE.0),
        };

        let set_ok = ImmSetConversionStatus(himc, new_conversion, sentence).as_bool();
        let _ = ImmReleaseContext(hwnd, himc);
        set_ok
    }
}

fn black_magic_is_english(hwnd: HWND) -> Option<bool> {
    unsafe {
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        if ime_hwnd.0.is_null() {
            return None;
        }
        let result = SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(IMC_GETCONVERSIONMODE),
            LPARAM(0),
        );
        let mode = result.0 as u32;
        let is_chinese = (mode & IME_CMODE_NATIVE.0) != 0;
        Some(!is_chinese)
    }
}

fn black_magic_set_mode(hwnd: HWND, mode: InputMode) -> bool {
    unsafe {
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        if ime_hwnd.0.is_null() {
            return false;
        }
        let bits = match mode {
            InputMode::English => IME_CMODE_ALPHANUMERIC.0,
            InputMode::Chinese => IME_CMODE_NATIVE.0,
        };
        let _ = SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(IMC_SETCONVERSIONMODE),
            LPARAM(bits as isize),
        );
        true
    }
}
