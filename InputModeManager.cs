#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

public class InputModeManager
{
    private readonly HashSet<string> engApps = new();
    private readonly HashSet<string> chnApps = new();

    public InputModeManager()
    {
        LoadConfig();
    }

    public void Handle(IntPtr hwnd)
    {
        string? exe = WinApi.GetProcessName(hwnd);
        string? title = WinApi.GetWindowTitle(hwnd);
        bool isEng = IMEController.IsEnglishMode();

        Console.WriteLine($"✅ 处理窗口: {title} ({exe}) → 当前状态: {(isEng ? "🔠 英文" : "🈶 中文")}");

        if (exe is null) return;

        // ✅ 简化判断逻辑：从配置中直接查找 exe 名字
        if (engApps.Contains(exe) && !isEng)
        {
            IMEController.SetToEnglish();
            Console.WriteLine("⏹ 已切换至 🔠 英文模式");
        }
        else if (chnApps.Contains(exe) && isEng)
        {
            IMEController.SetToChinese();
            Console.WriteLine("⏹ 已切换至 🈶 中文模式");
        }
    }

    private void LoadConfig()
    {
        if (!File.Exists(AppInputSettingsUI.ConfigPath))
        {
            Console.WriteLine("⚠️ 未找到输入法配置文件，跳过加载。");
            return;
        }

        try
        {
            var text = File.ReadAllText(AppInputSettingsUI.ConfigPath);
            var json = JsonSerializer.Deserialize<Dictionary<string, List<string>>>(text);

            if (json != null)
            {
                foreach (var app in json.GetValueOrDefault("ENG", new()))
                    engApps.Add(app);
                foreach (var app in json.GetValueOrDefault("CHINESE", new()))
                    chnApps.Add(app);
            }

            Console.WriteLine($"📘 配置加载成功：英文 {engApps.Count} 个，中文 {chnApps.Count} 个");
        }
        catch (Exception ex)
        {
            Console.WriteLine("❌ 加载输入法配置失败: " + ex.Message);
        }
    }
}
