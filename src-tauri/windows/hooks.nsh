; CWLD installer hooks.
;
; Why the installer said "Failed to kill CWLD" (round 0.10.1):
; Tauri's installer does not look for running cwld.exe processes. It asks the Windows Restart
; Manager which processes have the FILE cwld.exe in use, and asks Windows to shut them down.
; When that is a process CWLD cannot shut down (an antivirus scan, Windows Search, Explorer
; showing the icon, a backup tool), the installer stops, even though CWLD itself is closed.
; In the installer log this showed as "CWLD has stopped" followed by "Failed to kill CWLD".
;
; What these hooks do:
; 1. Before installing or uninstalling: end cwld.exe and its WebView2 processes and wait
;    until they are really gone (up to 15 seconds).
; 2. Replace Tauri's check by our own: stop only when cwld.exe is really still running.
;    If another program only has the file open, write in the install window which one it is,
;    move the old cwld.exe aside (Windows allows renaming a file in use) and carry on.

!macro CWLD_STOP_LEFTOVERS
  Push $0
  Push $1
  Push $2
  DetailPrint "Stopping CWLD and its WebView2 processes if they are still running"
  StrCpy $1 0
  ${Do}
    nsExec::Exec 'taskkill /F /T /IM cwld.exe'
    Pop $2
    Sleep 500
    ; find returns 0 when cwld.exe is still listed, 1 when it is gone.
    nsExec::Exec 'cmd /c tasklist /FI "IMAGENAME eq cwld.exe" /NH | find /I "cwld.exe"'
    Pop $0
    ${If} $0 != 0
      ${Break}
    ${EndIf}
    IntOp $1 $1 + 1
    ${If} $1 >= 30
      ${Break}
    ${EndIf}
    DetailPrint "Waiting for CWLD to close ($1)"
  ${Loop}
  ${If} $0 == 0
    DetailPrint "CWLD is still running after 15 seconds."
    nsExec::ExecToStack `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "Get-CimInstance Win32_Process -Filter \"Name='cwld.exe'\" | ForEach-Object { 'pid ' + $$_.ProcessId + ' ' + $$_.ExecutablePath }"`
    Pop $0
    Pop $2
    DetailPrint "Process found: $2"
  ${Else}
    DetailPrint "CWLD has stopped"
  ${EndIf}
  nsExec::Exec `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "foreach ($$i in 1..3) { Get-CimInstance Win32_Process | Where-Object { $$_.Name -eq 'msedgewebview2.exe' -and ($$_.CommandLine -like '*on3vz.cwld*' -or $$_.CommandLine -like '*webview-exe-name=cwld.exe*') } | ForEach-Object { Stop-Process -Id $$_.ProcessId -Force -ErrorAction SilentlyContinue }; Start-Sleep -Milliseconds 500 }"`
  Pop $0
  Sleep 500
  Pop $2
  Pop $1
  Pop $0
!macroend

; Writes a small PowerShell script that asks the Restart Manager which processes have a file open.
; Only used to show the name in the install window. It changes nothing.
!macro CWLD_WRITE_RM_SCRIPT
  Push $0
  FileOpen $0 "$TEMP\cwld-who-holds.ps1" w
  FileWrite $0 `param([string]$$Path)$\r$\n`
  FileWrite $0 `$$ErrorActionPreference = 'SilentlyContinue'$\r$\n`
  FileWrite $0 `$$src = @"$\r$\n`
  FileWrite $0 `using System;$\r$\n`
  FileWrite $0 `using System.Runtime.InteropServices;$\r$\n`
  FileWrite $0 `using System.Text;$\r$\n`
  FileWrite $0 `public static class CwldRm {$\r$\n`
  FileWrite $0 `  [StructLayout(LayoutKind.Sequential)]$\r$\n`
  FileWrite $0 `  struct RM_UNIQUE_PROCESS { public int dwProcessId; public System.Runtime.InteropServices.ComTypes.FILETIME ProcessStartTime; }$\r$\n`
  FileWrite $0 `  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]$\r$\n`
  FileWrite $0 `  struct RM_PROCESS_INFO {$\r$\n`
  FileWrite $0 `    public RM_UNIQUE_PROCESS Process;$\r$\n`
  FileWrite $0 `    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)] public string strAppName;$\r$\n`
  FileWrite $0 `    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)] public string strServiceShortName;$\r$\n`
  FileWrite $0 `    public int ApplicationType; public uint AppStatus; public uint TSSessionId;$\r$\n`
  FileWrite $0 `    [MarshalAs(UnmanagedType.Bool)] public bool bRestartable;$\r$\n`
  FileWrite $0 `  }$\r$\n`
  FileWrite $0 `  [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)] static extern int RmStartSession(out uint h, int f, StringBuilder k);$\r$\n`
  FileWrite $0 `  [DllImport("rstrtmgr.dll")] static extern int RmEndSession(uint h);$\r$\n`
  FileWrite $0 `  [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)] static extern int RmRegisterResources(uint h, uint nf, string[] f, uint na, IntPtr a, uint ns, string[] s);$\r$\n`
  FileWrite $0 `  [DllImport("rstrtmgr.dll")] static extern int RmGetList(uint h, out uint need, ref uint n, [In, Out] RM_PROCESS_INFO[] info, ref uint reasons);$\r$\n`
  FileWrite $0 `  public static string List(string path) {$\r$\n`
  FileWrite $0 `    uint h; var key = new StringBuilder(64);$\r$\n`
  FileWrite $0 `    if (RmStartSession(out h, 0, key) != 0) return "restart manager not available";$\r$\n`
  FileWrite $0 `    try {$\r$\n`
  FileWrite $0 `      if (RmRegisterResources(h, 1, new string[] { path }, 0, IntPtr.Zero, 0, null) != 0) return "could not register file";$\r$\n`
  FileWrite $0 `      uint need = 0, n = 0, reasons = 0;$\r$\n`
  FileWrite $0 `      int r = RmGetList(h, out need, ref n, null, ref reasons);$\r$\n`
  FileWrite $0 `      if (r == 0) return "nothing";$\r$\n`
  FileWrite $0 `      var info = new RM_PROCESS_INFO[need]; n = need;$\r$\n`
  FileWrite $0 `      if (RmGetList(h, out need, ref n, info, ref reasons) != 0) return "list failed";$\r$\n`
  FileWrite $0 `      var sb = new StringBuilder();$\r$\n`
  FileWrite $0 `      for (int i = 0; i < n; i++) sb.Append("pid " + info[i].Process.dwProcessId + " " + info[i].strAppName + (info[i].strServiceShortName.Length > 0 ? " (service " + info[i].strServiceShortName + ")" : "") + "; ");$\r$\n`
  FileWrite $0 `      return sb.ToString();$\r$\n`
  FileWrite $0 `    } finally { RmEndSession(h); }$\r$\n`
  FileWrite $0 `  }$\r$\n`
  FileWrite $0 `}$\r$\n`
  FileWrite $0 `"@$\r$\n`
  FileWrite $0 `Add-Type -TypeDefinition $$src$\r$\n`
  FileWrite $0 `$$out = [CwldRm]::List($$Path)$\r$\n`
  FileWrite $0 `$$names = @()$\r$\n`
  FileWrite $0 `foreach ($$m in [regex]::Matches($$out, 'pid (\d+)')) {$\r$\n`
  FileWrite $0 `  $$p = Get-CimInstance Win32_Process -Filter ("ProcessId=" + $$m.Groups[1].Value)$\r$\n`
  FileWrite $0 `  if ($$p) { $$names += ($$p.Name + " " + $$p.ExecutablePath) }$\r$\n`
  FileWrite $0 `}$\r$\n`
  FileWrite $0 `Write-Output ($$out + " " + ($$names -join "; "))$\r$\n`
  FileWrite $0 `$\r$\n`
  FileClose $0
  Pop $0
!macroend

; Our replacement of Tauri's CheckIfAppIsRunning (same name and arguments).
!macroundef CheckIfAppIsRunning
!macro CheckIfAppIsRunning executablePath productName
  !define CWLD_ID ${__LINE__}
  Push $0
  Push $1
  ; 1. Is cwld.exe really still running? Then stop, with a clear message.
  nsExec::Exec 'cmd /c tasklist /FI "IMAGENAME eq cwld.exe" /NH | find /I "cwld.exe"'
  Pop $0
  ${If} $0 == 0
    DetailPrint "CWLD is still running. Close it in Task Manager (Details tab, cwld.exe, End task) and run the installer again."
    Abort "CWLD is still running. Close it in Task Manager (Details tab, cwld.exe, End task) and run the installer again."
  ${EndIf}
  ; 2. Not running. If the old file is still held by another program, say which one and move it aside.
  ${If} ${FileExists} "${executablePath}"
    ClearErrors
    FileOpen $1 "${executablePath}" a
    ${If} ${Errors}
      DetailPrint "The old cwld.exe is in use by another program (CWLD itself is closed)."
      !insertmacro CWLD_WRITE_RM_SCRIPT
      nsExec::ExecToStack `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$TEMP\cwld-who-holds.ps1" -Path "${executablePath}"`
      Pop $0
      Pop $0
      DetailPrint "In use by: $0"
      Delete "$TEMP\cwld-who-holds.ps1"
      Delete "${executablePath}.old"
      ClearErrors
      Rename "${executablePath}" "${executablePath}.old"
      ${If} ${Errors}
        DetailPrint "Could not move the old cwld.exe aside. Windows may ask to retry, close the program named above first."
      ${Else}
        DetailPrint "Moved the old cwld.exe aside, it is removed later."
        Delete /REBOOTOK "${executablePath}.old"
      ${EndIf}
    ${Else}
      FileClose $1
    ${EndIf}
  ${EndIf}
  Pop $1
  Pop $0
  !undef CWLD_ID
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend
