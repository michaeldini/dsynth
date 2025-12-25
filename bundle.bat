@echo off
REM Bundle script for creating CLAP plugin bundle on Windows
REM Note: DSynth is CLAP-only (no longer supports VST3)

setlocal enabledelayedexpansion

set PLUGIN_NAME=DSynth
set BUILD_DIR=target\release
set BUNDLE_DIR=target\bundled

echo Building DSynth CLAP plugin...

REM Build the CLAP plugin
echo Building release binary...
cargo build --release --lib --features clap

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

REM Create bundle directories
echo Creating bundle structure...
if exist "%BUNDLE_DIR%" rmdir /s /q "%BUNDLE_DIR%"
mkdir "%BUNDLE_DIR%"

REM Create CLAP bundle
set CLAP_BUNDLE=%BUNDLE_DIR%\%PLUGIN_NAME%.clap
mkdir "%CLAP_BUNDLE%"

REM Copy and rename the dll for CLAP
copy "%BUILD_DIR%\dsynth.dll" "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap"

echo.
echo ✅ CLAP plugin bundle created successfully!
echo.
echo CLAP: %CLAP_BUNDLE%
echo.
echo To install:
echo   copy "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap" "%%COMMONPROGRAMFILES%%\CLAP\"
echo.
echo Or for per-user install:
echo   copy "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap" "%%LOCALAPPDATA%%\Programs\Common\CLAP\"
echo.

endlocal
