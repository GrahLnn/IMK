use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn get_foreground_process_name() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    get_process_name(hwnd)
}

pub fn get_process_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };

    let mut buf = vec![0u16; 1024];
    let len = unsafe { GetProcessImageFileNameW(handle, &mut buf) } as usize;
    let _ = unsafe { CloseHandle(handle) };
    if len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buf[..len]);
    Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}
