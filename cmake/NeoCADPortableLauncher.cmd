@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "APP_EXE=%SCRIPT_DIR%NeoCAD.exe"
set "FIXED_RUNTIME_DIR=%SCRIPT_DIR%webview2-fixed-runtime"

if not exist "%APP_EXE%" (
	echo NeoCAD.exe nao foi encontrado ao lado deste launcher.
	exit /b 1
)

if exist "%FIXED_RUNTIME_DIR%\msedgewebview2.exe" (
	icacls "%FIXED_RUNTIME_DIR%" /grant "*S-1-15-2-2:(OI)(CI)(RX)" >nul 2>nul
	icacls "%FIXED_RUNTIME_DIR%" /grant "*S-1-15-2-1:(OI)(CI)(RX)" >nul 2>nul
	set "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER=%FIXED_RUNTIME_DIR%"
)

"%APP_EXE%" %*
exit /b %errorlevel%
