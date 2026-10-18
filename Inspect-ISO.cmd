@echo off
call "%~dp0scripts\cargo.cmd" run --locked -p neopets-data -- %*
pause
