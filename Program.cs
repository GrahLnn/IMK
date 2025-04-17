using System;

class Program
{
    static void Main(string[] args)
    {
        Console.OutputEncoding = System.Text.Encoding.UTF8;
        Console.WriteLine("🚀 IMK 启动");

        var modeManager = new InputModeManager();
        var monitor = new WindowMonitor();

        monitor.OnWindowConfirmedChanged += hwnd => modeManager.Handle(hwnd);
        monitor.Start();

        System.Windows.Forms.Application.Run();
    }
}

