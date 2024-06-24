; Uncomment one of following lines, if you haven't checked "Add IDP include path to ISPPBuiltins.iss" option during IDP installation:
#pragma include __INCLUDE__ + ";" + ReadReg(HKLM, "Software\Mitrich Software\Inno Download Plugin", "InstallDir")
#pragma include __INCLUDE__ + ";" + "c:\lib\InnoDownloadPlugin"

[Setup]
AppName                = RecSOMs
AppVersion             = 1.0
DefaultDirName         = {pf}\RecSOMs
DefaultGroupName       = RecSOMs
Compression=lzma2
SolidCompression=yes
; Size of files to download:
ExtraDiskSpaceRequired = 1048576
OutputDir              = userdocs:Inno Setup Examples Output

#include <idp.iss>

[Files]
Source: "resources\*"; DestDir: "{app}\resources\"
Source: "sample_data\*"; DestDir: "{app}\sample_data\"
Source: "target\release\final-recurrent-soms.exe"; DestDir: "{app}"

[Icons]
Name: "{group}\{cm:UninstallProgram,RecSOMs}"; Filename: "{uninstallexe}"

[Tasks]
Name: StartAfterInstall; Description: Run application after install

[Run]
Filename: "{app}\final-recurrent-soms.exe"; Flags: shellexec skipifsilent nowait; Tasks: StartAfterInstall


