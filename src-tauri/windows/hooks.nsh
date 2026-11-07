; CWLD installer hooks. They run before the installer checks whether CWLD is still running.
;
; Two things can make the installer fail with "Failed to kill CWLD":
; 1. cwld.exe needs a few seconds to disappear after it is ended, because it holds the USB
;    serial port and the USB audio device. The installer used to check after half a second.
;    Here we end it and then wait, up to 15 seconds, until it is really gone.
; 2. WebView2 processes that draw the CWLD window can stay alive after CWLD itself has closed.
;    They are found by the data folder of CWLD (io.github.on3vz.cwld) or by the host name
;    cwld.exe in their command line, so WebView2 processes of other programs are left alone.

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
    DetailPrint "CWLD is still running after 15 seconds. Close it in Task Manager (Details tab, cwld.exe, End task) and run the installer again."
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

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend
