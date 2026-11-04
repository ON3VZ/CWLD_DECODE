; CWLD installer hooks. They run before the installer checks whether CWLD is still running.
;
; Windows Restart Manager counts every process that holds cwld.exe open. The WebView2
; processes that draw the CWLD window can stay alive after CWLD itself has closed, and the
; installer then fails with "Failed to kill CWLD". Here those leftovers are stopped first.
; They are found by the data folder of CWLD (io.github.on3vz.cwld) or by the host name
; cwld.exe in their command line, so WebView2 processes of other programs are left alone.

!macro CWLD_STOP_LEFTOVERS
  DetailPrint "Stopping CWLD and its WebView2 processes if they are still running"
  nsExec::Exec 'taskkill /F /T /IM cwld.exe'
  Pop $0
  nsExec::Exec `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "foreach ($$i in 1..3) { Get-CimInstance Win32_Process | Where-Object { $$_.Name -eq 'msedgewebview2.exe' -and ($$_.CommandLine -like '*on3vz.cwld*' -or $$_.CommandLine -like '*webview-exe-name=cwld.exe*') } | ForEach-Object { Stop-Process -Id $$_.ProcessId -Force -ErrorAction SilentlyContinue }; Start-Sleep -Milliseconds 500 }"`
  Pop $0
  Sleep 500
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CWLD_STOP_LEFTOVERS
!macroend
