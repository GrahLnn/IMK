use std::path::{Path, PathBuf};
use std::sync::Arc;

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMessageW, GetWindowLongPtrW, LoadIconW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, SetWindowLongPtrW, TrackPopupMenu, TranslateMessage, CREATESTRUCTW,
    GWLP_USERDATA, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, MSG, TPM_RIGHTBUTTON, WNDCLASSW,
    WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_NCCREATE, WM_RBUTTONUP, WS_OVERLAPPED,
};

const TRAY_MESSAGE: u32 = WM_APP + 1;
const TRAY_ID: u32 = 1;

const ID_OPEN_SETTINGS: usize = 1001;
const ID_OPEN_CONFIG: usize = 1002;
const ID_OPEN_CONFIG_DIR: usize = 1003;
const ID_TOGGLE_AUTOSTART: usize = 1004;
const ID_EXIT: usize = 1005;

struct TrayState {
    config_path: PathBuf,
    on_open_settings: Arc<dyn Fn() + Send + Sync>,
    on_exit: Arc<dyn Fn() + Send + Sync>,
}

pub struct Tray {
    hwnd: HWND,
    state_ptr: *mut TrayState,
}

impl Tray {
    pub fn new() -> Self {
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("IMKTrayWnd");

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            let state = Box::new(TrayState {
                config_path: default_config_path(),
                on_open_settings: Arc::new(|| {}),
                on_exit: Arc::new(|| {}),
            });
            let state_ptr = Box::into_raw(state);

            let hwnd = CreateWindowExW(
                Default::default(),
                class_name,
                w!(""),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                HWND(0 as _),
                None,
                hinstance,
                Some(state_ptr as _),
            )
            .unwrap_or_default();

            if !hwnd.0.is_null() {
                let icon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
                let mut nid = NOTIFYICONDATAW::default();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = TRAY_ID;
                nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
                nid.uCallbackMessage = TRAY_MESSAGE;
                nid.hIcon = icon;
                write_tip(&mut nid, "IMK 输入法管理器");
                let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);
            }

            Self { hwnd, state_ptr }
        }
    }

    pub fn set_callbacks<F, G>(&mut self, on_open_settings: F, on_exit: G)
    where
        F: Fn() + Send + Sync + 'static,
        G: Fn() + Send + Sync + 'static,
    {
        unsafe {
            if !self.state_ptr.is_null() {
                (*self.state_ptr).on_open_settings = Arc::new(on_open_settings);
                (*self.state_ptr).on_exit = Arc::new(on_exit);
            }
        }
    }

    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND(0 as _), 0, 0).0 != 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.0.is_null() {
                let mut nid = NOTIFYICONDATAW::default();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = self.hwnd;
                nid.uID = TRAY_ID;
                let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
            }
            if !self.hwnd.0.is_null() {
                let _ = DestroyWindow(self.hwnd);
            }
            if !self.state_ptr.is_null() {
                let _ = Box::from_raw(self.state_ptr);
            }
        }
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        let create: &CREATESTRUCTW = &*(lparam.0 as *const CREATESTRUCTW);
        let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayState;
    if !state_ptr.is_null() {
        let state = &*state_ptr;
        match msg {
            TRAY_MESSAGE => {
                let event = lparam.0 as u32;
                if event == WM_RBUTTONUP {
                    show_tray_menu(hwnd);
                } else if event == WM_LBUTTONDBLCLK {
                    (state.on_open_settings)();
                }
                return LRESULT(0);
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xffff) as usize;
                handle_command(id, state);
                return LRESULT(0);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                return LRESULT(0);
            }
            _ => {}
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn show_tray_menu(hwnd: HWND) {
    unsafe {
        let _ = SetForegroundWindow(hwnd);
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);

        let menu = CreatePopupMenu().unwrap_or_default();
        build_menu(menu);
        let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, None);
        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyMenu(menu);
    }
}

fn build_menu(menu: windows::Win32::UI::WindowsAndMessaging::HMENU) {
    unsafe {
        let _ = AppendMenuW(menu, MF_STRING, ID_OPEN_SETTINGS, w!("打开设置"));
        let _ = AppendMenuW(menu, MF_STRING, ID_OPEN_CONFIG, w!("打开配置文件"));
        let _ = AppendMenuW(menu, MF_STRING, ID_OPEN_CONFIG_DIR, w!("打开配置文件夹"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, w!(""));
        let _ = AppendMenuW(menu, MF_STRING, ID_TOGGLE_AUTOSTART, w!("开机自启"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, w!(""));
        let _ = AppendMenuW(menu, MF_STRING, ID_EXIT, w!("退出"));
    }
}

fn handle_command(id: usize, state: &TrayState) {
    match id {
        ID_OPEN_SETTINGS => (state.on_open_settings)(),
        ID_OPEN_CONFIG => open_path(&state.config_path),
        ID_OPEN_CONFIG_DIR => {
            if let Some(dir) = state.config_path.parent() {
                open_path(dir);
            }
        }
        ID_TOGGLE_AUTOSTART => toggle_autostart(),
        ID_EXIT => (state.on_exit)(),
        _ => {}
    }
}

fn open_path(path: impl AsRef<Path>) {
    let _ = std::process::Command::new("explorer").arg(path.as_ref()).spawn();
}

fn default_config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("IMK").join("default_input_config.json")
}

fn startup_folder() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
}

fn startup_shortcut_path() -> PathBuf {
    startup_folder().join("IMK 输入法管理器.lnk")
}

fn startup_bat_path() -> PathBuf {
    startup_folder().join("IMK 输入法管理器.bat")
}

fn is_autostart_enabled() -> bool {
    startup_shortcut_path().exists() || startup_bat_path().exists()
}

fn toggle_autostart() {
    let enable = !is_autostart_enabled();
    let _ = set_autostart(enable);
}

fn set_autostart(enable: bool) -> std::io::Result<()> {
    let shortcut = startup_shortcut_path();
    let bat = startup_bat_path();

    if enable {
        let exe = std::env::current_exe()?;
        let script = format!(
            "$WshShell = New-Object -comObject WScript.Shell\n$Shortcut = $WshShell.CreateShortcut('{}')\n$Shortcut.TargetPath = '{}'\n$Shortcut.WorkingDirectory = '{}'\n$Shortcut.Description = 'IMK 输入法管理器'\n$Shortcut.Save()\n",
            shortcut.display(),
            exe.display(),
            exe.parent().unwrap_or(Path::new("")).display()
        );
        let _ = std::process::Command::new("powershell.exe")
            .arg("-Command")
            .arg(script)
            .spawn();
    } else {
        if shortcut.exists() {
            let _ = std::fs::remove_file(&shortcut);
        }
        if bat.exists() {
            let _ = std::fs::remove_file(&bat);
        }
    }

    Ok(())
}

fn write_tip(nid: &mut NOTIFYICONDATAW, tip: &str) {
    let mut wide: Vec<u16> = tip.encode_utf16().collect();
    wide.push(0);
    let max = nid.szTip.len().min(wide.len());
    nid.szTip[..max].copy_from_slice(&wide[..max]);
}
