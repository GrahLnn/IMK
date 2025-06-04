#nullable enable

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Windows.Forms;
using System.Reflection;

public class AppInputSettingsUI : Form
{
    private TextBox appListBox;
    private TextBox engInputBox;
    private TextBox chnInputBox;
    private Button saveButton;

    public static readonly string ConfigPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "IMK", "default_input_config.json");

    public AppInputSettingsUI()
    {
        Text = "IMK 应用语言设置";
        Width = 600;
        Height = 500;

        appListBox = new TextBox
        {
            Multiline = true,
            ReadOnly = true,
            ScrollBars = ScrollBars.Vertical,
            Width = 550,
            Height = 120,
            Left = 20,
            Top = 20
        };

        engInputBox = new TextBox
        {
            Multiline = true,
            Width = 250,
            Height = 120,
            Left = 20,
            Top = 160
        };

        chnInputBox = new TextBox
        {
            Multiline = true,
            Width = 250,
            Height = 120,
            Left = 300,
            Top = 160
        };

        Label engLabel = new Label
        {
            Text = "🔠 英文应用（逗号分隔）",
            Left = 20,
            Top = 140,
            Width = 200
        };
        Label chnLabel = new Label
        {
            Text = "🈶 中文应用（逗号分隔）",
            Left = 300,
            Top = 140,
            Width = 200
        };

        saveButton = new Button
        {
            Text = "保存设置",
            Left = 240,
            Top = 300,
            Width = 100
        };
        saveButton.Click += OnSaveClicked;

        Controls.Add(appListBox);
        Controls.Add(engInputBox);
        Controls.Add(chnInputBox);
        Controls.Add(engLabel);
        Controls.Add(chnLabel);
        Controls.Add(saveButton);

        LoadAppList();
        LoadExistingConfig();
    }

    private void LoadAppList()
    {
        var processes = Process.GetProcesses()
            .Where(p => p.MainWindowHandle != IntPtr.Zero)
            .Select(p => p.ProcessName + ".exe")
            .Distinct()
            .OrderBy(p => p)
            .ToList();

        appListBox.Lines = processes.ToArray();
    }

    private void LoadExistingConfig()
    {
        if (File.Exists(ConfigPath))
        {
            try
            {
                var json = File.ReadAllText(ConfigPath);
                var config = JsonSerializer.Deserialize<Dictionary<string, List<string>>>(json);
                if (config != null)
                {
                    if (config.ContainsKey("ENG"))
                        engInputBox.Text = string.Join(", ", config["ENG"]);
                    if (config.ContainsKey("CHINESE"))
                        chnInputBox.Text = string.Join(", ", config["CHINESE"]);
                }
            }
            catch { }
        }
    }

    private void OnSaveClicked(object? sender, EventArgs e)
    {
        var engList = engInputBox.Text.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var chnList = chnInputBox.Text.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

        var json = new Dictionary<string, List<string>>
        {
            ["ENG"] = engList.ToList(),
            ["CHINESE"] = chnList.ToList()
        };

        string dir = Path.GetDirectoryName(ConfigPath)!;
        if (!Directory.Exists(dir))
            Directory.CreateDirectory(dir);

        File.WriteAllText(ConfigPath, JsonSerializer.Serialize(json, new JsonSerializerOptions
        {
            WriteIndented = true
        }));

        MessageBox.Show("配置已保存到\n" + ConfigPath, "保存成功", MessageBoxButtons.OK, MessageBoxIcon.Information);
    }

    public static bool ConfigExists()
    {
        return File.Exists(ConfigPath);
    }
}

// IMK 系统托盘应用
public class IMKTrayApp : ApplicationContext
{
    private readonly WindowMonitor monitor;
    private readonly InputModeManager modeManager;
    private readonly NotifyIcon trayIcon;

    public IMKTrayApp()
    {
        try
        {
            string logPath = Path.Combine(Path.GetTempPath(), "IMK_Debug.log");
            File.AppendAllText(logPath, "开始创建IMKTrayApp...\n");

            // 初始化输入法管理器
            File.AppendAllText(logPath, "创建InputModeManager...\n");
            modeManager = new InputModeManager();
            
            File.AppendAllText(logPath, "创建WindowMonitor...\n");
            monitor = new WindowMonitor();
            monitor.OnWindowConfirmedChanged += hwnd => modeManager.Handle(hwnd);
            
            File.AppendAllText(logPath, "启动WindowMonitor...\n");
            monitor.Start();

            File.AppendAllText(logPath, "创建系统托盘图标...\n");
            // 创建系统托盘图标
            trayIcon = new NotifyIcon()
            {
                Icon = SystemIcons.Application,
                Visible = true,
                Text = "IMK 输入法管理器"
            };

            File.AppendAllText(logPath, "创建右键菜单...\n");
            // 创建右键菜单
            var contextMenu = new ContextMenuStrip();
            
            // 打开设置菜单项
            var settingsMenuItem = new ToolStripMenuItem("打开设置", null, (_, _) =>
            {
                var settingsForm = new AppInputSettingsUI();
                settingsForm.ShowDialog();
            });

            // 打开配置文件菜单项
            var openConfigMenuItem = new ToolStripMenuItem("打开配置文件", null, (_, _) =>
            {
                try
                {
                    if (File.Exists(AppInputSettingsUI.ConfigPath))
                    {
                        Process.Start(new ProcessStartInfo
                        {
                            FileName = AppInputSettingsUI.ConfigPath,
                            UseShellExecute = true
                        });
                    }
                    else
                    {
                        MessageBox.Show("配置文件不存在，请先进行设置。", "提示", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    }
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"无法打开配置文件：{ex.Message}", "错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
            });

            // 打开配置文件目录菜单项
            var openConfigDirMenuItem = new ToolStripMenuItem("打开配置文件夹", null, (_, _) =>
            {
                try
                {
                    string configDir = Path.GetDirectoryName(AppInputSettingsUI.ConfigPath)!;
                    if (!Directory.Exists(configDir))
                        Directory.CreateDirectory(configDir);
                    
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = configDir,
                        UseShellExecute = true
                    });
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"无法打开配置文件夹：{ex.Message}", "错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
            });

            // 开机自启菜单项
            var autoStartMenuItem = new ToolStripMenuItem();
            UpdateAutoStartMenuItem(autoStartMenuItem);
            autoStartMenuItem.Click += (_, _) =>
            {
                bool currentState = IsAutoStartEnabled();
                SetAutoStart(!currentState);
                UpdateAutoStartMenuItem(autoStartMenuItem);
            };

            // 分隔线
            var separator1 = new ToolStripSeparator();
            var separator2 = new ToolStripSeparator();

            // 退出菜单项
            var exitMenuItem = new ToolStripMenuItem("退出", null, (_, _) =>
            {
                trayIcon.Visible = false;
                monitor?.Stop();
                Application.Exit();
            });

            File.AppendAllText(logPath, "添加菜单项...\n");
            // 添加所有菜单项
            contextMenu.Items.Add(settingsMenuItem);
            contextMenu.Items.Add(openConfigMenuItem);
            contextMenu.Items.Add(openConfigDirMenuItem);
            contextMenu.Items.Add(separator1);
            contextMenu.Items.Add(autoStartMenuItem);
            contextMenu.Items.Add(separator2);
            contextMenu.Items.Add(exitMenuItem);

            trayIcon.ContextMenuStrip = contextMenu;

            // 双击托盘图标打开设置
            trayIcon.DoubleClick += (_, _) =>
            {
                var settingsForm = new AppInputSettingsUI();
                settingsForm.ShowDialog();
            };

            File.AppendAllText(logPath, "检查配置文件...\n");
            // 如果配置文件不存在，首次运行时自动打开设置
            if (!AppInputSettingsUI.ConfigExists())
            {
                File.AppendAllText(logPath, "配置文件不存在，显示欢迎界面...\n");
                MessageBox.Show("欢迎使用 IMK 输入法管理器！\n首次运行需要进行配置。", "欢迎", MessageBoxButtons.OK, MessageBoxIcon.Information);
                var settingsForm = new AppInputSettingsUI();
                settingsForm.ShowDialog();
            }

            File.AppendAllText(logPath, "IMKTrayApp 创建完成！\n");
        }
        catch (Exception ex)
        {
            string logPath = Path.Combine(Path.GetTempPath(), "IMK_Debug.log");
            string errorMsg = $"IMKTrayApp 创建失败: {ex.Message}\n堆栈跟踪:\n{ex.StackTrace}\n";
            
            try
            {
                File.AppendAllText(logPath, errorMsg);
            }
            catch { }

            MessageBox.Show($"系统托盘应用创建失败！\n\n错误信息：{ex.Message}\n\n日志文件位置：{logPath}", 
                          "IMK 错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
            throw; // 重新抛出异常
        }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            trayIcon?.Dispose();
            monitor?.Stop();
        }
        base.Dispose(disposing);
    }

    // 开机自启相关方法
    private string GetStartupShortcutPath()
    {
        string startupFolder = Environment.GetFolderPath(Environment.SpecialFolder.Startup);
        return Path.Combine(startupFolder, "IMK 输入法管理器.lnk");
    }

    private bool IsAutoStartEnabled()
    {
        string shortcutPath = GetStartupShortcutPath();
        string batPath = shortcutPath.Replace(".lnk", ".bat");
        return File.Exists(shortcutPath) || File.Exists(batPath);
    }

    private void SetAutoStart(bool enable)
    {
        try
        {
            string shortcutPath = GetStartupShortcutPath();
            string batPath = shortcutPath.Replace(".lnk", ".bat");
            
            if (enable)
            {
                // 创建快捷方式
                CreateShortcut(shortcutPath);
                MessageBox.Show("开机自启已启用", "设置成功", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
            else
            {
                // 删除快捷方式
                if (File.Exists(shortcutPath))
                {
                    File.Delete(shortcutPath);
                }
                if (File.Exists(batPath))
                {
                    File.Delete(batPath);
                }
                MessageBox.Show("开机自启已禁用", "设置成功", MessageBoxButtons.OK, MessageBoxIcon.Information);
            }
        }
        catch (Exception ex)
        {
            MessageBox.Show($"设置开机自启失败：{ex.Message}", "错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }

    private void CreateShortcut(string shortcutPath)
    {
        try
        {
            // 获取当前程序的完整路径
            string exePath = Assembly.GetExecutingAssembly().Location;
            if (exePath.EndsWith(".dll"))
            {
                // 如果是dll，需要找到对应的exe文件
                exePath = exePath.Replace(".dll", ".exe");
            }

            // 使用PowerShell创建快捷方式
            string script = $@"
$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut('{shortcutPath}')
$Shortcut.TargetPath = '{exePath}'
$Shortcut.WorkingDirectory = '{Path.GetDirectoryName(exePath)}'
$Shortcut.Description = 'IMK 输入法管理器'
$Shortcut.Save()
";

            var startInfo = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-Command \"{script}\"",
                UseShellExecute = false,
                CreateNoWindow = true,
                WindowStyle = ProcessWindowStyle.Hidden
            };

            using (var process = Process.Start(startInfo))
            {
                process?.WaitForExit();
            }
        }
        catch (Exception)
        {
            // 如果PowerShell方法失败，使用COM接口
            CreateShortcutWithCOM(shortcutPath);
        }
    }

    private void CreateShortcutWithCOM(string shortcutPath)
    {
        // 简化版本：直接复制exe到启动文件夹（不是最佳做法，但作为备选方案）
        string exePath = Assembly.GetExecutingAssembly().Location;
        if (exePath.EndsWith(".dll"))
        {
            exePath = exePath.Replace(".dll", ".exe");
        }
        
        string startupFolder = Path.GetDirectoryName(shortcutPath)!;
        string targetExePath = Path.Combine(startupFolder, "IMK.exe");
        
        // 如果目标位置没有exe文件，复制一份
        if (!File.Exists(targetExePath))
        {
            File.Copy(exePath, targetExePath, true);
        }
        
        // 创建一个简单的批处理文件作为启动脚本
        string batPath = Path.Combine(startupFolder, "IMK 输入法管理器.bat");
        File.WriteAllText(batPath, $@"@echo off
cd /d ""{Path.GetDirectoryName(exePath)}""
start """" ""{exePath}""
");
        
        // 将批处理文件重命名为快捷方式路径（作为标记）
        if (File.Exists(batPath))
        {
            File.Move(batPath, shortcutPath.Replace(".lnk", ".bat"));
        }
    }

    private void UpdateAutoStartMenuItem(ToolStripMenuItem menuItem)
    {
        bool currentState = IsAutoStartEnabled();
        menuItem.Text = currentState ? "✓ 禁用开机自启" : "启用开机自启";
        menuItem.Checked = currentState;
    }
}