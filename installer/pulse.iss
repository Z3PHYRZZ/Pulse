[Setup]
AppName=Pulse
AppVersion=2.0.3
AppPublisher=Z3PHYRZZ
AppPublisherURL=https://github.com/Z3PHYRZZ/Pulse
DefaultDirName={autopf}\Pulse
DefaultGroupName=Pulse
UninstallDisplayIcon={app}\pulse.exe
SetupIconFile=..\icons\pulse.ico
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
Source: "..\icons\pulse.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Pulse"; Filename: "{app}\pulse.exe"
Name: "{group}\Pulse (minimized)"; Filename: "{app}\pulse.exe"; Parameters: "--minimized"
Name: "{autodesktop}\Pulse"; Filename: "{app}\pulse.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop icon"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\pulse.exe"; Description: "Launch Pulse"; Flags: nowait postinstall skipifsilent

