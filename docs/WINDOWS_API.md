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
| Tray | `Shell_NotifyIconW` + message-only `HWND_MESSAGE` | Callback routed through `WM_USER + 1`. |

All raw Win32 calls are confined to `src/collectors`, `src/windows`, and the API smoke tests.
`@;
$files['installer\pulse.iss']=@'
[Setup]
AppName=Pulse
AppVersion=0.1.0
AppPublisher=Z3PHYRZZ
AppPublisherURL=https://github.com/Z3PHYRZZ/Pulse
DefaultDirName={autopf}\Pulse
DefaultGroupName=Pulse
UninstallDisplayIcon={app}\pulse.exe
OutputBaseFilename=PulseSetup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\LICENSE

[Files]
Source: "..\target\release\pulse.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\config.default.toml"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Pulse"; Filename: "{app}\pulse.exe"
Name: "{group}\Pulse (minimized)"; Filename: "{app}\pulse.exe"; Parameters: "--minimized"
Name: "{autodesktop}\Pulse"; Filename: "{app}\pulse.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop icon"; GroupDescription: "Additional icons:"
Name: "startupicon"; Description: "Start Pulse with Windows"; GroupDescription: "Startup:"

[Run]
Filename: "{app}\pulse.exe"; Description: "Launch Pulse"; Flags: nowait postinstall skipifsilent
