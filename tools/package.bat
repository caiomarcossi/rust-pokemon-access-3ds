@echo off
rem Runs package.ps1 without changing the machine's execution policy; arguments pass through (-SkipTests).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0package.ps1" %*
