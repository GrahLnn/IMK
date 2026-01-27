# IMK Rust 重写设计文档

**日期:** 2026-01-27

## 目标
用 Rust + gpui 重写 IMK 输入法管理器，保持现有功能与配置格式，同时引入多策略 IME 切换（IMM32 优先、黑科技兜底），提升可维护性。

## 架构概览
- **core**：配置模型、进程匹配、输入模式决策、重试与延迟策略。
- **platform_win**：Win32 API 绑定与实现（Shell hook、前台窗口、IME 切换、托盘、快捷方式）。
- **ui**：gpui 设置窗口（进程列表、英文/中文列表编辑、保存）。
- **tray**：托盘图标与菜单命令（设置、配置文件/目录、自启、退出）。
- **service**：窗口稳定确认与切换调度。

## 数据流
Shell hook 触发前台窗口变化 → 稳定确认（500ms 稳定 + 150ms 延迟） → 读取进程名 → 依据配置映射 ENG/CHINESE → 选择目标输入模式 → IME 多策略切换 → 结果日志。

## IME 切换策略
1) **IMM32 正规路径**：`ImmGetContext` + `ImmGetConversionStatus` + `ImmSetConversionStatus`，并释放上下文。
2) **黑科技兜底**：`ImmGetDefaultIMEWnd` + `SendMessage(WM_IME_CONTROL)`。
3) **重试与延迟**：短延迟 + 最多 N 次重试，确保状态稳定。

## 配置与存储
- 路径：`%APPDATA%\IMK\default_input_config.json`
- 格式：`{ "ENG": ["notepad.exe"], "CHINESE": ["wechat.exe"] }`
- UI 与托盘均支持打开/编辑。

## 错误处理与日志
- Win32 调用失败不崩溃，记录错误并继续运行。
- IME 切换失败回退策略继续尝试。

## 测试策略
- 单元测试：配置解析、匹配逻辑。
- 手动测试：托盘菜单、窗口切换、IME 切换在 Win11 25H2 + 微软拼音。
