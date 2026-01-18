@echo off
REM Bundle script for creating DSynth Kick CLAP plugin on Windows

setlocal enabledelayedexpansion

set PLUGIN_NAME=DSynthKick
if not "%TARGET%"=="" (
    set BUILD_DIR=target\%TARGET%\release
) else (
    set BUILD_DIR=target\release
)
set BUNDLE_DIR=target\bundled

echo Building DSynth Kick CLAP plugin...

REM Build the CLAP plugin
echo Building release binary...
if not "%TARGET%"=="" (
    cargo build --release --lib --no-default-features --features kick-clap --target %TARGET%
) else (
    cargo build --release --lib --no-default-features --features kick-clap
)

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

REM Create bundle directories
if exist "%BUNDLE_DIR%" (mkdir "%BUNDLE_DIR%" >nul 2>&1) else (mkdir "%BUNDLE_DIR%")

REM Copy and rename the dll for CLAP (flat file)
copy "%BUILD_DIR%\dsynth.dll" "%BUNDLE_DIR%\%PLUGIN_NAME%.clap"

echo.
echo ✅ CLAP plugin created: %BUNDLE_DIR%\%PLUGIN_NAME%.clap
echo.
echo To install:
echo   copy "%BUNDLE_DIR%\%PLUGIN_NAME%.clap" "%%COMMONPROGRAMFILES%%\CLAP\"
echo.
echo Or for per-user install:
echo   copy "%BUNDLE_DIR%\%PLUGIN_NAME%.clap" "%%LOCALAPPDATA%%\Programs\Common\CLAP\"
echo.

endlocal