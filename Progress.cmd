@echo off
powershell -NoProfile -File "%~dp0scripts\progress.ps1"
if not defined NEOPETS_NO_PAUSE pause
