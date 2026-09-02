; NeuralDisk Windows installer.
;
; Compiled with Inno Setup's ISCC.exe. The version and path to the compiled
; binary are passed in from CI:
;   ISCC.exe /DMyAppVersion=1.0.0 /DMyAppBinary=path\to\neuraldisk.exe packaging\windows\neuraldisk.iss
;
; Ollama is never bundled - this only detects it and, with the user's
; yes/no consent, opens the official download page / kicks off the model
; pull. See packaging/macos/scripts/postinstall for the equivalent macOS
; flow and the reasoning behind not bundling Ollama ourselves.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef MyAppBinary
  #define MyAppBinary "..\..\target\release\neuraldisk.exe"
#endif

#define MyAppName "NeuralDisk"
#define MyAppPublisher "NeuralDisk"
#define MyAppURL "https://github.com/k-aksha/neural-disk"
#define MyAppExeName "neuraldisk.exe"

[Setup]
AppId={{6C6E6572-616C-4469-736B-4E657572616C}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\NeuralDisk
DefaultGroupName=NeuralDisk
DisableProgramGroupPage=yes
; Relative to this .iss file, so CI (which invokes ISCC from the repo root)
; ends up with Output\ at the repo root rather than under packaging\windows\.
OutputDir=..\..\Output
OutputBaseFilename=NeuralDisk-Setup-{#MyAppVersion}
SetupIconFile=..\..\neuraldisk\icons\neuraldisk_logo_flag.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
; No code-signing certificate is available for this build - Windows
; SmartScreen will show an "unknown publisher" warning until one is added.

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#MyAppBinary}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\NeuralDisk"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,NeuralDisk}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\NeuralDisk"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,NeuralDisk}"; Flags: nowait postinstall skipifsilent

[Code]
const
  OllamaDownloadUrl = 'https://ollama.com/download';

function OllamaExePath(): String;
begin
  Result := ExpandConstant('{localappdata}\Programs\Ollama\ollama.exe');
end;

function IsOllamaInstalled(): Boolean;
var
  ResultCode: Integer;
begin
  // Ollama's own Windows installer places it under %LOCALAPPDATA%\Programs\Ollama.
  // Fall back to checking PATH via `where` in case it was installed elsewhere
  // (e.g. a portable/manual install).
  if FileExists(OllamaExePath()) then
  begin
    Result := True;
    exit;
  end;
  Result := Exec('cmd.exe', '/c where ollama >nul 2>&1', '', SW_HIDE,
    ewWaitUntilTerminated, ResultCode) and (ResultCode = 0);
end;

procedure OfferOllamaModelPull();
var
  ResultCode: Integer;
begin
  if MsgBox('Download the default AI model (llama3.1, about 4.7 GB) now?' + #13#10 +
    'This can take a while depending on your connection - a console window will ' +
    'open and the download will continue there after you click Yes.',
    mbConfirmation, MB_YESNO) = IDYES then
  begin
    // /k keeps the window open after the command finishes so the user can see
    // the result; ewNoWait so the installer itself doesn't block on a
    // multi-gigabyte download.
    Exec('cmd.exe', '/k ollama pull llama3.1', '', SW_SHOW, ewNoWait, ResultCode);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
  OllamaReady: Boolean;
begin
  if CurStep <> ssPostInstall then
    exit;

  OllamaReady := IsOllamaInstalled();

  if not OllamaReady then
  begin
    if MsgBox('NeuralDisk''s optional AI copilot needs a local Ollama server.' + #13#10 +
      'Install Ollama now? (Every other feature works fine without it - you ' +
      'can always do this later.)', mbConfirmation, MB_YESNO) = IDYES then
    begin
      ShellExec('open', OllamaDownloadUrl, '', '', SW_SHOWNORMAL, ewNoWait, ResultCode);
      MsgBox('Continue the Ollama installer in your browser, then come back and ' +
        'run NeuralDisk once it finishes - the AI copilot will detect it ' +
        'automatically.', mbInformation, MB_OK);
      // Ollama's installer runs asynchronously in the browser/downloaded exe,
      // so we can't reliably know it finished in time to offer the model
      // pull within this same run - the app itself will report the missing
      // model if the user tries the copilot before pulling one.
      exit;
    end
    else
    begin
      MsgBox('No problem - NeuralDisk works fully without it. Install Ollama ' +
        'later from ' + OllamaDownloadUrl + ' to enable the AI copilot.',
        mbInformation, MB_OK);
      exit;
    end;
  end;

  OfferOllamaModelPull();
end;
