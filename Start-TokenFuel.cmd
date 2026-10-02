@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Start-TokenFuel.ps1" %*
exit /b %errorlevel%
