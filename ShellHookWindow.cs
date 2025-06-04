using System;
using System.Runtime.InteropServices;
using System.Text;

public class ShellHookWindow : IDisposable
{
    private const int HSHELL_WINDOWACTIVATED = 4;
    private const string SHELLHOOK = "SHELLHOOK";

    private IntPtr _hwnd;
    private IntPtr _msgShellHook;
    private WndProcDelegate _wndProcDelegate;
    private IntPtr _hInstance;
    private string _className = "IMKShellHookWnd";

    private Action<IntPtr> _onWindowActivated;

    public ShellHookWindow(Action<IntPtr> onWindowActivated)
    {
        _onWindowActivated = onWindowActivated;
        _hInstance = GetModuleHandle(null);
    }

    public void Start()
    {
        _wndProcDelegate = new WndProcDelegate(WndProc);

        WNDCLASS wc = new WNDCLASS
        {
            lpfnWndProc = Marshal.GetFunctionPointerForDelegate(_wndProcDelegate),
            hInstance = _hInstance,
            lpszClassName = _className
        };

        RegisterClass(ref wc);

        _hwnd = CreateWindowEx(
            0,
            _className,
            string.Empty,
            0,
            0, 0, 0, 0,
            IntPtr.Zero,
            IntPtr.Zero,
            _hInstance,
            IntPtr.Zero
        );

        _msgShellHook = RegisterWindowMessage(SHELLHOOK);
        RegisterShellHookWindow(_hwnd);
    }

    public void Stop()
    {
        if (_hwnd != IntPtr.Zero)
        {
            UnregisterShellHookWindow(_hwnd);
        }
    }

    private IntPtr WndProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam)
    {
        if (msg == _msgShellHook)
        {
            if (wParam.ToInt32() == HSHELL_WINDOWACTIVATED)
            {
                _onWindowActivated?.Invoke(lParam);
            }
        }
        return DefWindowProc(hWnd, msg, wParam, lParam);
    }

    public void Dispose()
    {
        if (_hwnd != IntPtr.Zero)
        {
            UnregisterShellHookWindow(_hwnd);
            DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }
    }

    #region WinAPI

    private delegate IntPtr WndProcDelegate(
    IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam
);

    [StructLayout(LayoutKind.Sequential)]
    private struct WNDCLASS
    {
        public uint style;
        public IntPtr lpfnWndProc;
        public int cbClsExtra;
        public int cbWndExtra;
        public IntPtr hInstance;
        public IntPtr hIcon;
        public IntPtr hCursor;
        public IntPtr hbrBackground;
        public string lpszMenuName;
        public string lpszClassName;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern ushort RegisterClass(ref WNDCLASS lpWndClass);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CreateWindowEx(
        int dwExStyle,
        string lpClassName,
        string lpWindowName,
        int dwStyle,
        int x, int y, int nWidth, int nHeight,
        IntPtr hWndParent,
        IntPtr hMenu,
        IntPtr hInstance,
        IntPtr lpParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool DestroyWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern IntPtr DefWindowProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern bool RegisterShellHookWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern bool UnregisterShellHookWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern IntPtr RegisterWindowMessage(string lpString);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    private static extern IntPtr GetModuleHandle(string lpModuleName);

    #endregion
}
