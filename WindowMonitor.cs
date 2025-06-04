#nullable enable

using System;
using System.Threading;
using System.Threading.Tasks;

public class WindowMonitor : IDisposable
{
    public event Action<IntPtr>? OnWindowConfirmedChanged;

    private IntPtr _lastConfirmedHwnd = IntPtr.Zero;
    private ShellHookWindow _shellHook;
    private CancellationTokenSource? _confirmationCts;
    private readonly object _lock = new object(); // Lock for critical sections if needed, especially around _confirmationCts

    // --- Configuration ---
    // How long the window handle must remain stable before confirming (milliseconds)
    private const int StabilizeDurationMs = 500;
    // How often to check the window handle during stabilization (milliseconds)
    private const int CheckIntervalMs = 50;
    // Extra delay after stabilization before raising event (for IME state etc.)
    private const int PostStabilizationDelayMs = 150;
    // --------------------

    public WindowMonitor()
    {
        _shellHook = new ShellHookWindow(OnShellWindowActivated);
    }

    public void Start()
    {
        _shellHook.Start();
    }

    public void Stop()
    {
        lock (_lock)
        {
            _confirmationCts?.Cancel();
        }
        _shellHook?.Stop();
    }

    // Note: Event handlers are often async void, which is acceptable here.
    private async void OnShellWindowActivated(IntPtr activatedHwnd) // activatedHwnd might be transient
    {
        // Cancel any previous confirmation attempt that might be running
        lock (_lock)
        {
            _confirmationCts?.Cancel();
            _confirmationCts = new CancellationTokenSource();
        }

        // Capture the cancellation token for this specific attempt
        var currentCts = _confirmationCts;
        var token = currentCts.Token;

        try
        {
            // Start the stabilization check asynchronously
            await Task.Run(async () => // Run the check logic on a background thread
            {
                await Task.Delay(CheckIntervalMs, token); // Initial small delay

                IntPtr stableHwnd = IntPtr.Zero;
                IntPtr lastCheckedHwnd = IMEController.GetForegroundWindow();
                DateTime stableStartTime = DateTime.UtcNow;

                while (!token.IsCancellationRequested)
                {
                    // Give OS some time slice
                    await Task.Delay(CheckIntervalMs, token);

                    IntPtr currentHwnd = IMEController.GetForegroundWindow();

                    if (currentHwnd == IntPtr.Zero) // Ignore invalid handles
                    {
                        stableStartTime = DateTime.UtcNow; // Reset timer
                        lastCheckedHwnd = IntPtr.Zero;
                        continue;
                    }

                    if (currentHwnd == lastCheckedHwnd)
                    {
                        // Window handle hasn't changed since last check
                        if ((DateTime.UtcNow - stableStartTime).TotalMilliseconds >= StabilizeDurationMs)
                        {
                            // Window has been stable for the required duration
                            stableHwnd = currentHwnd;
                            break; // Confirmed stable
                        }
                        // else: Still stable, but not long enough yet, continue checking
                    }
                    else
                    {
                        // Window handle changed, reset the stability timer
                        lastCheckedHwnd = currentHwnd;
                        stableStartTime = DateTime.UtcNow;
                    }
                } // End while loop

                // --- Check if successfully stabilized and not cancelled ---
                if (!token.IsCancellationRequested && stableHwnd != IntPtr.Zero)
                {
                    // Check if this stable window is different from the last one we notified about
                    bool notify = false;
                    lock (_lock) // Protect access to _lastConfirmedHwnd
                    {
                        if (stableHwnd != _lastConfirmedHwnd)
                        {
                            _lastConfirmedHwnd = stableHwnd;
                            notify = true;
                        }
                    }

                    if (notify)
                    {
                        // *** Add extra delay specifically for IME state stabilization ***
                        await Task.Delay(PostStabilizationDelayMs, token);

                        // Final check before raising event (in case of cancellation during delay)
                        if (!token.IsCancellationRequested)
                        {
                            // It's generally safer to invoke events on the original context
                            // if subscribers expect it (e.g., UI updates), but for this
                            // background task, invoking directly might be okay.
                            // If issues arise, consider using SynchronizationContext.Post or similar.
                            OnWindowConfirmedChanged?.Invoke(stableHwnd);
                            Console.WriteLine($"Window Confirmed: {stableHwnd} (Stable)"); // Debugging
                        }
                    }
                }

            }, token); // Pass token to Task.Run
        }
        catch (OperationCanceledException)
        {
            // Expected when a new activation happens quickly, cancelling the previous check.
            // Console.WriteLine("Confirmation cancelled."); // Optional debug log
        }
        catch (Exception ex)
        {
            // Log unexpected errors during confirmation
            Console.WriteLine($"Error during window confirmation: {ex.Message}");
        }
        finally
        {
             // Clean up the CTS if it's the one we created.
             // Need careful handling if multiple activations happen extremely fast.
             lock(_lock)
             {
                 if (_confirmationCts == currentCts)
                 {
                     _confirmationCts.Dispose();
                     _confirmationCts = null;
                 }
                 // else: a newer activation already created a new CTS, leave it alone.
             }
        }
    }

    public void Dispose()
    {
        lock (_lock)
        {
            _confirmationCts?.Cancel();
            _confirmationCts?.Dispose();
            _confirmationCts = null;
        }
        _shellHook?.Dispose();
        GC.SuppressFinalize(this); // Prevent finalizer call if Dispose is called
    }

    // Optional: Finalizer as a safety net, though explicit Dispose is preferred.
    ~WindowMonitor()
    {
        Dispose();
    }
}


// --- Keep IMEController and ShellHookWindow as they were (assuming ShellHookWindow is correct) ---
// (IMEController code from your example)
// public static class IMEController { ... }
// (Assuming ShellHookWindow implementation exists and correctly calls the provided callback)
// internal class ShellHookWindow : IDisposable { ... }
