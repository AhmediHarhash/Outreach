@echo off
echo ========================================
echo     Voice Copilot - Quick Build
echo ========================================
echo.

echo Building release version...
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Build FAILED!
    pause
    exit /b 1
)

echo.
echo ========================================
echo     Build Successful!
echo ========================================
echo.
echo Executable: target\release\voice-copilot.exe
echo.
echo To run: target\release\voice-copilot.exe
echo.

set /p RUN="Run now? (y/n): "
if /i "%RUN%"=="y" (
    start "" "target\release\voice-copilot.exe"
)
