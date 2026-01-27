# IMK Rust 重写 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 用 Rust + gpui 重写 IMK，保持托盘/配置/自启/IME 切换行为一致，并加入 IMM32 优先 + 黑科技兜底策略。

**Architecture:** 模块化拆分 core/platform/ui/tray/service；IME 切换采用策略链；窗口稳定确认保持现有延迟策略。

**Tech Stack:** Rust, gpui, windows-rs (Win32 bindings), serde_json, anyhow

---

### Task 1: 初始化 Rust 工程骨架

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/core/mod.rs`
- Create: `src/platform_win/mod.rs`
- Create: `src/ui/mod.rs`
- Create: `src/tray/mod.rs`
- Create: `src/service/mod.rs`

**Step 1: 写一个 failing test（占位）**

```rust
#[test]
fn placeholder_fail() {
    assert!(false, "placeholder");
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL with "placeholder"

**Step 3: 替换为最小可过测试**

```rust
#[test]
fn placeholder_pass() {
    assert!(true);
}
```

**Step 4: 再次运行测试**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add Cargo.toml src
 git commit -m "chore: scaffold rust project"
```

---

### Task 2: 配置模型与解析

**Files:**
- Create: `src/core/config.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/config_parse.rs`

**Step 1: 写 failing test**

```rust
use imk::core::config::InputConfig;

#[test]
fn parse_config_eng_and_chinese() {
    let json = r#"{\"ENG\":[\"notepad.exe\"],\"CHINESE\":[\"wechat.exe\"]}"#;
    let cfg = InputConfig::from_json_str(json).unwrap();
    assert!(cfg.eng.contains("notepad.exe"));
    assert!(cfg.chinese.contains("wechat.exe"));
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing type/functions

**Step 3: 实现最小解析**

```rust
pub struct InputConfig { pub eng: std::collections::HashSet<String>, pub chinese: std::collections::HashSet<String> }
impl InputConfig { pub fn from_json_str(s: &str) -> anyhow::Result<Self> { /* serde_json */ } }
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/core tests/config_parse.rs
 git commit -m "feat: add config parsing"
```

---

### Task 3: 进程匹配与模式决策

**Files:**
- Create: `src/core/mode_manager.rs`
- Modify: `src/core/mod.rs`
- Create: `tests/mode_decision.rs`

**Step 1: 写 failing test**
```rust
use imk::core::mode_manager::{InputMode, ModeDecision};

#[test]
fn decide_switch_to_eng_when_in_list() {
    let decision = ModeDecision::for_process("notepad.exe", false, &["notepad.exe"], &[]);
    assert_eq!(decision, Some(InputMode::English));
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing types

**Step 3: 实现最小逻辑**

```rust
pub enum InputMode { English, Chinese }
impl ModeDecision { pub fn for_process(...) -> Option<InputMode> { /* set logic */ } }
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/core tests/mode_decision.rs
 git commit -m "feat: add mode decision logic"
```

---

### Task 4: Win32 绑定与前台窗口/进程名

**Files:**
- Create: `src/platform_win/winapi.rs`
- Modify: `src/platform_win/mod.rs`
- Create: `tests/winapi_stub.rs` (只做编译验证)

**Step 1: 写 failing test（编译）**
```rust
use imk::platform_win::winapi::get_foreground_process_name;

#[test]
fn foreground_process_name_compiles() {
    let _ = get_foreground_process_name();
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing function

**Step 3: 实现最小 WinAPI 封装**

```rust
pub fn get_foreground_process_name() -> Option<String> { /* GetForegroundWindow + GetWindowThreadProcessId + OpenProcess + QueryFullProcessImageNameW */ }
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/platform_win tests/winapi_stub.rs
 git commit -m "feat: add winapi process lookup"
```

---

### Task 5: IME 多策略切换（IMM32 + 黑科技兜底）

**Files:**
- Create: `src/platform_win/ime.rs`
- Modify: `src/platform_win/mod.rs`
- Create: `tests/ime_strategy.rs` (只做编译验证)

**Step 1: 写 failing test（编译）**
```rust
use imk::platform_win::ime::{ImeController, InputMode};

#[test]
fn ime_controller_compiles() {
    let _ = ImeController::new();
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing types

**Step 3: 实现策略链**

```rust
pub enum InputMode { English, Chinese }
pub struct ImeController { /* ... */ }
impl ImeController { pub fn set_mode(&self, mode: InputMode) -> bool { /* IMM32 then fallback */ } }
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/platform_win tests/ime_strategy.rs
 git commit -m "feat: add ime strategy chain"
```

---

### Task 6: Shell Hook + 稳定窗口确认服务

**Files:**
- Create: `src/service/window_monitor.rs`
- Modify: `src/service/mod.rs`
- Create: `tests/window_monitor.rs` (编译验证)

**Step 1: 写 failing test（编译）**
```rust
use imk::service::window_monitor::WindowMonitor;

#[test]
fn window_monitor_compiles() {
    let _ = WindowMonitor::new();
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing types

**Step 3: 实现 Shell hook 与稳定确认**

```rust
// 使用 RegisterShellHookWindow + WndProc，稳定确认 500ms + 150ms 延迟
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/service tests/window_monitor.rs
 git commit -m "feat: add window monitor service"
```

---

### Task 7: 托盘与菜单

**Files:**
- Create: `src/tray/tray.rs`
- Modify: `src/tray/mod.rs`

**Step 1: 写 failing test（编译）**
```rust
use imk::tray::tray::Tray;

#[test]
fn tray_compiles() {
    let _ = Tray::new();
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing types

**Step 3: 实现托盘与菜单逻辑**

```rust
// 包含：打开设置/配置文件/配置目录/开机自启/退出
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/tray
 git commit -m "feat: add tray menu"
```

---

### Task 8: gpui 设置界面

**Files:**
- Create: `src/ui/settings.rs`
- Modify: `src/ui/mod.rs`

**Step 1: 写 failing test（编译）**
```rust
use imk::ui::settings::SettingsWindow;

#[test]
fn settings_window_compiles() {
    let _ = SettingsWindow::new();
}
```

**Step 2: 运行测试确认失败**
Run: `cargo test`
Expected: FAIL missing types

**Step 3: 实现 gpui 界面**

```rust
// 左侧显示进程列表，右侧 ENG/CHINESE 文本框，保存按钮
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/ui
 git commit -m "feat: add gpui settings window"
```

---

### Task 9: 应用入口与集成

**Files:**
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

**Step 1: 写 failing test（编译）**
```rust
#[test]
fn app_entry_compiles() {
    assert!(true);
}
```

**Step 2: 运行测试确认失败/通过**
Run: `cargo test`
Expected: PASS（确认编译）

**Step 3: 集成服务与托盘**

```rust
// 初始化配置 + 监控服务 + 托盘 + 设置窗口
```

**Step 4: 运行测试确认通过**
Run: `cargo test`
Expected: PASS

**Step 5: Commit**
```bash
git add src/main.rs src/lib.rs
 git commit -m "feat: wire up app entry"
```

---

### Task 10: 手动测试清单与文档更新

**Files:**
- Modify: `README.md`

**Step 1: 更新 README 构建与运行说明**

**Step 2: 记录手动测试清单**

**Step 3: Commit**
```bash
git add README.md
 git commit -m "docs: update readme for rust build"
```
