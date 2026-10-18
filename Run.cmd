@echo off
call "%~dp0scripts\cargo.cmd" run --locked -p neopets-app -- %*
if errorlevel 1 pause
