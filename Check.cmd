@echo off
call "%~dp0scripts\cargo.cmd" test --locked
if errorlevel 1 goto fail
call "%~dp0scripts\cargo.cmd" check --locked -p neopets-app
if errorlevel 1 goto fail
powershell -NoProfile -File "%~dp0scripts\audit-source.ps1"
if errorlevel 1 goto fail
powershell -NoProfile -File "%~dp0scripts\audit-specs.ps1"
if errorlevel 1 goto fail
echo All checks passed.
if not defined NEOPETS_NO_PAUSE pause
exit /b 0
:fail
echo Check failed.
if not defined NEOPETS_NO_PAUSE pause
exit /b 1
