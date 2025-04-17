#nullable enable

using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

public static class WinApi
{
    public static string? GetWindowTitle(IntPtr hWnd)
    {
        var buffer = new StringBuilder(512);
        if (GetWindowText(hWnd, buffer, buffer.Capacity) > 0)
            return buffer.ToString();
        return null;
    }

    public static string? GetClassName(IntPtr hWnd)
    {
        var buffer = new StringBuilder(256);
        if (GetClassName(hWnd, buffer, buffer.Capacity) > 0)
            return buffer.ToString();
        return null;
    }

    public static string? GetProcessName(IntPtr hWnd)
    {
        _ = GetWindowThreadProcessId(hWnd, out int pid);
        try
        {
            return Process.GetProcessById(pid).ProcessName + ".exe";
        }
        catch
        {
            return null;
        }
    }

    [DllImport("user32.dll")]
    private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    private static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);

    [DllImport("user32.dll")]
    private static extern int GetWindowThreadProcessId(IntPtr hWnd, out int lpdwProcessId);
}
