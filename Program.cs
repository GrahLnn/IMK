using System;
using System.Windows.Forms;
using System.IO;

class Program
{
    [STAThread]
    static void Main(string[] args)
    {
        try
        {
            // 创建日志文件来调试问题
            string logPath = Path.Combine(Path.GetTempPath(), "IMK_Debug.log");
            File.WriteAllText(logPath, $"IMK 程序启动: {DateTime.Now}\n");

            // 在Debug模式下启用控制台输出
#if DEBUG
            Console.WriteLine("=== IMK 输入法控制器启动 ===");
            Console.WriteLine($"启动时间: {DateTime.Now}");
            Console.WriteLine($"日志文件位置: {logPath}");
            Console.WriteLine("正在初始化应用程序...");
#endif

            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            File.AppendAllText(logPath, "正在创建托盘应用...\n");

#if DEBUG
            Console.WriteLine("正在创建系统托盘应用...");
#endif

            // 直接启动系统托盘应用，不显示配置界面
            Application.Run(new IMKTrayApp());
            
            File.AppendAllText(logPath, "程序正常退出\n");

#if DEBUG
            Console.WriteLine("程序正常退出");
            Console.WriteLine("按任意键退出...");
            Console.ReadKey();
#endif
        }
        catch (Exception ex)
        {
            // 如果出现异常，显示错误信息并记录到日志
            string logPath = Path.Combine(Path.GetTempPath(), "IMK_Debug.log");
            string errorMsg = $"程序启动失败: {ex.Message}\n堆栈跟踪:\n{ex.StackTrace}\n";
            
            try
            {
                File.AppendAllText(logPath, errorMsg);
            }
            catch { }

#if DEBUG
            Console.WriteLine($"错误: {ex.Message}");
            Console.WriteLine($"详细错误信息已保存到: {logPath}");
            Console.WriteLine("按任意键退出...");
            Console.ReadKey();
#endif

            MessageBox.Show($"IMK 程序启动失败！\n\n错误信息：{ex.Message}\n\n日志文件位置：{logPath}", 
                          "IMK 启动错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}

