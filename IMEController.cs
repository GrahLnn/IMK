using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class IMEController
{
    private const uint WM_IME_CONTROL = 0x0283;
    private const int IMC_GETCONVERSIONMODE = 0x0001;
    private const int IMC_SETCONVERSIONMODE = 0x0002;

    private const int IME_CMODE_NATIVE = 0x1;
    private const int IME_CMODE_ALPHANUMERIC = 0x0;

    private const int MAX_RETRY = 50;


    public static IntPtr GetIMEWindow(IntPtr hwnd)
    {
        return ImmGetDefaultIMEWnd(hwnd);
    }

    public static bool IsEnglishMode()
    {
        IntPtr hwnd = GetForegroundWindow();
        IntPtr imeHwnd = GetIMEWindow(hwnd);
        if (imeHwnd == IntPtr.Zero)
            return false;

        IntPtr result = SendMessage(imeHwnd, WM_IME_CONTROL, IMC_GETCONVERSIONMODE, IntPtr.Zero);
        int mode = result.ToInt32();
        bool isChinese = (mode & IME_CMODE_NATIVE) != 0;
        return !isChinese;
    }

    public static bool SetToEnglish()
    {
        return Retry(() =>
        {
            IntPtr hwnd = GetForegroundWindow();
            IntPtr imeHwnd = GetIMEWindow(hwnd);
            SendMessage(imeHwnd, WM_IME_CONTROL, IMC_SETCONVERSIONMODE, new IntPtr(IME_CMODE_ALPHANUMERIC));
            Thread.Sleep(30);
            return IsEnglishMode();
        });
    }

    public static bool SetToChinese()
    {
        return Retry(() =>
        {
            IntPtr hwnd = GetForegroundWindow();
            IntPtr imeHwnd = GetIMEWindow(hwnd);
            SendMessage(imeHwnd, WM_IME_CONTROL, IMC_SETCONVERSIONMODE, new IntPtr(IME_CMODE_NATIVE));
            Thread.Sleep(30);
            return !IsEnglishMode();
        });
    }

    private static bool Retry(Func<bool> func, int retry = MAX_RETRY)
    {
        for (int i = 0; i < retry; i++)
        {
            if (func())
            {
                Thread.Sleep(100); // 状态稳定后再返回
                return true;
            }
            Thread.Sleep(50);
        }
        return false;
    }

    #region Native API

    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [DllImport("imm32.dll")]
    private static extern IntPtr ImmGetDefaultIMEWnd(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, int wParam, IntPtr lParam);

    #endregion
}
