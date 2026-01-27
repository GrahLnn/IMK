use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow,
    GetMessageW, GetWindowLongPtrW, PostMessageW, PostQuitMessage, RegisterClassW,
    RegisterShellHookWindow, RegisterWindowMessageW, SetWindowLongPtrW, TranslateMessage,
    CREATESTRUCTW, GWLP_USERDATA, MSG, WNDCLASSW, WM_CLOSE, WM_DESTROY, WM_NCCREATE,
    WS_OVERLAPPED,
};

const HSHELL_WINDOWACTIVATED: usize = 4;
const STABILIZE_DURATION_MS: u64 = 500;
const CHECK_INTERVAL_MS: u64 = 50;
const POST_STABILIZATION_DELAY_MS: u64 = 150;

struct HookState {
    shellhook_msg: u32,
    callback: Arc<dyn Fn(HWND) + Send + Sync>,
    generation: Arc<AtomicU64>,
}

pub struct WindowMonitor {
    callback: Arc<dyn Fn(HWND) + Send + Sync>,
    generation: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    hwnd: Arc<Mutex<Option<isize>>>,
    thread: Option<JoinHandle<()>>,
}

impl WindowMonitor {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(|_| {}),
            generation: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            hwnd: Arc::new(Mutex::new(None)),
            thread: None,
        }
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(HWND) + Send + Sync + 'static,
    {
        self.callback = Arc::new(callback);
    }

    pub fn start(&mut self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let callback = self.callback.clone();
        let generation = self.generation.clone();
        let running = self.running.clone();
        let hwnd_slot = self.hwnd.clone();

        let handle = thread::spawn(move || {
            unsafe {
                let hinstance = GetModuleHandleW(None).unwrap_or_default();
                let class_name = w!("IMKShellHookWnd");

                let wnd_class = WNDCLASSW {
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: hinstance.into(),
                    lpszClassName: class_name,
                    ..Default::default()
                };

                let _ = RegisterClassW(&wnd_class);

                let shellhook_msg = RegisterWindowMessageW(w!("SHELLHOOK"));
                let state = Box::new(HookState {
                    shellhook_msg,
                    callback,
                    generation,
                });
                let state_ptr = Box::into_raw(state);

                let hwnd = match CreateWindowExW(
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
                ) {
                    Ok(hwnd) => hwnd,
                    Err(_) => {
                        running.store(false, Ordering::SeqCst);
                        let _ = Box::from_raw(state_ptr);
                        return;
                    }
                };

                let _ = RegisterShellHookWindow(hwnd);

                {
                    let mut slot = hwnd_slot.lock().unwrap();
                    *slot = Some(hwnd.0 as isize);
                }

                let mut msg = MSG::default();
                while running.load(Ordering::SeqCst) {
                    let ret = GetMessageW(&mut msg, HWND(0 as _), 0, 0);
                    if ret.0 == 0 {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        self.thread = Some(handle);
    }

    pub fn stop(&mut self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }
        if let Some(raw) = self.hwnd.lock().unwrap().take() {
            unsafe {
                let hwnd = HWND(raw as *mut core::ffi::c_void);
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for WindowMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn schedule_confirmation(
    callback: Arc<dyn Fn(HWND) + Send + Sync>,
    generation: Arc<AtomicU64>,
) {
    let gen = generation.fetch_add(1, Ordering::SeqCst) + 1;
    thread::spawn(move || {
        let mut last = unsafe { GetForegroundWindow() };
        let mut stable_start = Instant::now();

        loop {
            thread::sleep(Duration::from_millis(CHECK_INTERVAL_MS));
            if generation.load(Ordering::SeqCst) != gen {
                return;
            }

            let current = unsafe { GetForegroundWindow() };
            if current.0.is_null() {
                stable_start = Instant::now();
                last = current;
                continue;
            }

            if current.0 == last.0 {
                if stable_start.elapsed() >= Duration::from_millis(STABILIZE_DURATION_MS) {
                    break;
                }
            } else {
                last = current;
                stable_start = Instant::now();
            }
        }

        if generation.load(Ordering::SeqCst) != gen {
            return;
        }

        thread::sleep(Duration::from_millis(POST_STABILIZATION_DELAY_MS));
        if generation.load(Ordering::SeqCst) != gen {
            return;
        }

        callback(last);
    });
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        let create: &CREATESTRUCTW = &*(lparam.0 as *const CREATESTRUCTW);
        let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }

    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut HookState;
    if !state_ptr.is_null() {
        let state = &*state_ptr;
        if msg == state.shellhook_msg && wparam.0 as usize == HSHELL_WINDOWACTIVATED {
            schedule_confirmation(state.callback.clone(), state.generation.clone());
            return LRESULT(0);
        }
    }

    match msg {
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            let ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut HookState;
            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
