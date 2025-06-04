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

            // 注意：WinExe应用程序没有控制台，所以不能设置Console.OutputEncoding
            // Console.OutputEncoding = System.Text.Encoding.UTF8;

            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            File.AppendAllText(logPath, "正在创建托盘应用...\n");

            // 直接启动系统托盘应用，不显示配置界面
            Application.Run(new IMKTrayApp());
            
            File.AppendAllText(logPath, "程序正常退出\n");
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

            MessageBox.Show($"IMK 程序启动失败！\n\n错误信息：{ex.Message}\n\n日志文件位置：{logPath}", 
                          "IMK 启动错误", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}

