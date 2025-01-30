[Setup]
AppName=GEMA_Launcher
AppVersion=1.0
DefaultDirName={pf}\GEMA_Launcher
; Place the setup output EXE in the same folder as the script:
OutputDir=.
OutputBaseFilename=GEMA_Launcher_Setup
UninstallDisplayIcon={app}\GEMA_Launcher.exe
Compression=lzma
SolidCompression=yes
 
[Files]
; Notice we just write the filenames/folders because
; they are in the SAME directory as this .iss file.
Source: "GEMA_Launcher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.pdf";         DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*";          DestDir: "{app}\assets"; Flags: recursesubdirs
 
[Icons]
Name: "{commondesktop}\GEMA_Launcher"; Filename: "{app}\GEMA_Launcher.exe"
 
[Code]
procedure DeleteSourceFolder;
var
  SourcePath: string;
  UserChoice: Integer;
begin
  // One way to get the folder containing the *running installer*:
  SourcePath := ExtractFilePath(ExpandConstant('{srcexe}'));
 
  // Ask the user if they want to delete the original folder
  UserChoice := MsgBox('Do you want to delete the original folder (' + SourcePath + ')?', mbConfirmation, MB_YESNO);
 
  if UserChoice = IDYES then
  begin
    if DirExists(SourcePath) then
    begin
      DelTree(SourcePath, True, True, True);
      MsgBox('The folder has been successfully deleted.', mbInformation, MB_OK);
    end
    else
      MsgBox('Folder could not be found.', mbError, MB_OK);
  end;
end;
 
[PostInstall]
; Option to start the application after installation
Filename: "{app}\GEMA_Launcher.exe"; Description: "Launch GEMA_Launcher"; Flags: nowait postinstall
 
; Offer to delete original folder if user agrees
Filename: "{code:DeleteSourceFolder}"; Description: "Delete original folder"; Flags: postinstall unchecked