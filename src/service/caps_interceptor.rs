use std::mem::size_of;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_LSHIFT, VK_RSHIFT, VK_SHIFT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::core::caps_logic::{decide_caps_action, should_turn_caps_off_on_shift, CapsAction};

const CAPS_TAP_DELAY_MS: u64 = 50;

#[derive(Default)]
struct HookState {
    caps_pressed: bool,
    caps_at: Option<Instant>,
}

static HOOK_STATE: OnceLock<Mutex<HookState>> = OnceLock::new();
static SIMULATING: AtomicBool = AtomicBool::new(false);

fn hook_state() -> &'static Mutex<HookState> {
    HOOK_STATE.get_or_init(|| Mutex::new(HookState::default()))
}

pub struct CapsInterceptor {
    running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
    thread: Option<JoinHandle<()>>,
}

impl CapsInterceptor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_id: Arc::new(AtomicU32::new(0)),
            thread: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = self.running.clone();
        let thread_id = self.thread_id.clone();

        let handle = thread::spawn(move || unsafe {
            thread_id.store(GetCurrentThreadId(), Ordering::SeqCst);
            {
                let mut state = hook_state().lock().unwrap();
                state.caps_pressed = false;
                state.caps_at = None;
            }

            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let hook =
                match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), Some(hinstance.into()), 0)
                {
                    Ok(hook) => hook,
                    Err(_) => {
                        running.store(false, Ordering::SeqCst);
                        return;
                    }
                };
            if hook.is_invalid() {
                running.store(false, Ordering::SeqCst);
                return;
            }

            let mut msg = MSG::default();
            while running.load(Ordering::SeqCst) {
                let ret = GetMessageW(&mut msg, None, 0, 0);
                if ret.0 == 0 {
                    break;
                }
            }

            let _ = UnhookWindowsHookEx(hook);
        });

        self.thread = Some(handle);
    }

    pub fn stop(&mut self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }

        let tid = self.thread_id.load(Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CapsInterceptor {
    fn drop(&mut self) {
        self.stop();
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    panic::catch_unwind(AssertUnwindSafe(|| {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let msg = wparam.0 as u32;
        let is_shift = matches!(
            kb.vkCode,
            code if code == VK_SHIFT.0 as u32
                || code == VK_LSHIFT.0 as u32
                || code == VK_RSHIFT.0 as u32
        );

        if (kb.flags.0 & LLKHF_INJECTED.0) != 0 || SIMULATING.load(Ordering::SeqCst) {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        if is_shift {
            if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN)
                && should_turn_caps_off_on_shift(is_caps_on())
            {
                spawn_action(CapsAction::SetCaps(false));
            }
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        if kb.vkCode != VK_CAPITAL.0 as u32 {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        let mut state = hook_state().lock().unwrap();

        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            if !state.caps_pressed {
                state.caps_pressed = true;
                state.caps_at = Some(Instant::now());
            }
            return LRESULT(1);
        }

        if msg == WM_KEYUP || msg == WM_SYSKEYUP {
            if state.caps_pressed {
                state.caps_pressed = false;
                let held_ms = state
                    .caps_at
                    .take()
                    .map(|t| t.elapsed().as_secs_f64() * 1000.0)
                    .unwrap_or(0.0);
                let action = decide_caps_action(held_ms);
                spawn_action(action);
            }
            return LRESULT(1);
        }

        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }))
    .unwrap_or_else(|_| unsafe { CallNextHookEx(None, code, wparam, lparam) })
}

fn spawn_action(action: CapsAction) {
    thread::spawn(move || {
        SIMULATING.store(true, Ordering::SeqCst);
        match action {
            CapsAction::SwitchIme => simulate_ctrl_space(),
            CapsAction::SetCaps(on) => set_caps(on),
        }
        thread::sleep(Duration::from_millis(CAPS_TAP_DELAY_MS));
        SIMULATING.store(false, Ordering::SeqCst);
    });
}

fn is_caps_on() -> bool {
    unsafe { (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0 }
}

fn set_caps(on: bool) {
    if is_caps_on() == on {
        return;
    }
    simulate_key_press(VK_CAPITAL);
}

fn simulate_ctrl_space() {
    let inputs = [
        key_input(VK_CONTROL, false),
        key_input(VK_SPACE, false),
        key_input(VK_SPACE, true),
        key_input(VK_CONTROL, true),
    ];
    unsafe {
        let _ = SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

fn simulate_key_press(vk: VIRTUAL_KEY) {
    let inputs = [key_input(vk, false), key_input(vk, true)];
    unsafe {
        let _ = SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

fn key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
