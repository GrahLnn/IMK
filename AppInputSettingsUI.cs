#nullable enable

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Windows.Forms;

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

    // [STAThread]
    // static void Main()
    // {
    //     Application.EnableVisualStyles();

    //     if (!ConfigExists())
    //     {
    //         Application.Run(new AppInputSettingsUI());
    //     }
    //     else
    //     {
    //         Application.Run(new SilentContextMenuApp()); // 后续你可替换为托盘程序主入口
    //     }
    // }
}

// 示例静默应用类（后续替换为真正托盘 UI）
public class SilentContextMenuApp : ApplicationContext
{
    private readonly WindowMonitor monitor;
    private readonly InputModeManager modeManager;
    private readonly NotifyIcon trayIcon;

    public SilentContextMenuApp()
    {
        modeManager = new InputModeManager();
        monitor = new WindowMonitor();
        monitor.OnWindowConfirmedChanged += hwnd => modeManager.Handle(hwnd);
        monitor.Start();

        trayIcon = new NotifyIcon()
        {
            Icon = SystemIcons.Application,
            Visible = true,
            Text = "IMK 输入法管理器",
            ContextMenuStrip = new ContextMenuStrip()
        };

        trayIcon.ContextMenuStrip.Items.Add("打开设置", null, (_, _) =>
        {
            new AppInputSettingsUI().ShowDialog();
        });
        trayIcon.ContextMenuStrip.Items.Add("退出", null, (_, _) =>
        {
            trayIcon.Visible = false;
            Application.Exit();
        });
    }
}