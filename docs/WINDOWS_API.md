# Pulse — Windows API Notes

| Feature | API | Notes |
|---|---|---|
| CPU usage | PDH `\Processor(_Total)\% Processor Time` | English counter names avoid locale-dependent paths. |
| CPU frequency | Registry `~MHz` + PDH `% Processor Performance` | Approximate effective clock. |
| CPU temperature | Not a standard PDH metric | Optional vendor/WMI/LHM collector only. |
| RAM | `GlobalMemoryStatusEx` | Uses `MEMORYSTATUSEX.dwLength` set to `size_of`. |
| Processes | `CreateToolhelp32Snapshot`, `Process32FirstW/NextW` | Snapshot handle closed through `CloseHandle`. |
| Process CPU | `GetProcessTimes` deltas | Kernel+user FILETIME divided by wall time and logical core count. |
| Process RAM | `GetProcessMemoryInfo` | `WorkingSetSize`. |
| GPU | NVML dynamic load + `CreateDXGIFactory1` | NVML is optional; DXGI reports adapter description and dedicated VRAM. |
| Disk throughput | PDH `LogicalDisk(_Total)` read/write bytes/sec | Rates are already per second. |
| Disk capacity | `GetLogicalDriveStringsW`, `GetDiskFreeSpaceExW` | Double-null terminated drive string buffer. |
| Network | PDH `Network Interface(*)` bytes sent/received/sec | Wildcard instances are summed by the counter. |
| Uptime | `GetTickCount64` | Milliseconds since boot. |
| OS version | Registry `HKLM\...\CurrentVersion` | `ProductName`, `CurrentBuild`. |
| Startup | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | `REG_SZ` with quoted executable and `--minimized`. |
| Tray | Tauri `TrayIconBuilder` (muda/tray-icon) | One tray owned by the Tauri shell. Show/hide/exit are menu commands; double-click shows the window. The old native `Shell_NotifyIconW` tray was removed. |
| Close | `WM_CLOSE` subclass + Tauri `CloseRequested` | `src-tauri/close.rs` stores one prev-wndproc per HWND. Close calls `prevent_close()` then `app.exit(0)` — full exit, no `ExitProcess` short-circuit. |

All raw Win32 calls are confined to `pulse-core/src/collectors`, `pulse-core/src/windows`,
`src-tauri/src/close.rs`, and the collector smoke tests in `pulse-core/tests/`.
The frontend in `frontend/` never calls Windows APIs directly; it talks to the shell
through Tauri commands and the `pulse://metrics` event.
