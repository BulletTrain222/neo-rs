@echo off
setlocal
cd /d "%~dp0.."
where cargo >nul 2>&1
if errorlevel 1 (
  echo Rust is missing. Install Rust with rustup and the Visual Studio C++ Build Tools.
  exit /b 1
)
rem Initialize the installed MSVC toolchain without altering the user's global environment.
if defined VSCMD_VER goto build
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" (
  for /f "usebackq tokens=*" %%i in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSROOT=%%i"
)
if not defined VSROOT (
  echo Visual Studio C++ Build Tools not found. See README.md.
  exit /b 1
)
call "%VSROOT%\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 exit /b 1
:build
cargo %*
exit /b %errorlevel%
