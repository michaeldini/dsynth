@echo off
REM Bundle script for creating CLAP plugin bundle on Windows

setlocal enabledelayedexpansion

set PLUGIN_NAME=DSynth
if not "%TARGET%"=="" (
    set BUILD_DIR=target\%TARGET%\release
) else (
    set BUILD_DIR=target\release
)
set BUNDLE_DIR=target\bundled

echo Building DSynth CLAP plugin...

REM Build the CLAP plugin
echo Building release binary...
if not "%TARGET%"=="" (
    cargo build --release --lib --no-default-features --features clap --target %TARGET%
) else (
    cargo build --release --lib --no-default-features --features clap
)

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

REM Create bundle directories
echo Creating bundle structure...
if exist "%BUNDLE_DIR%" rmdir /s /q "%BUNDLE_DIR%"
mkdir "%BUNDLE_DIR%"

REM Copy and rename the dll for CLAP (flat file)
copy "%BUILD_DIR%\dsynth.dll" "%BUNDLE_DIR%\%PLUGIN_NAME%.clap"

echo.
echo ✅ CLAP plugin bundle created successfully!
echo.
echo CLAP: %BUNDLE_DIR%\%PLUGIN_NAME%.clap
echo.
echo To install:
echo   copy "%BUNDLE_DIR%\%PLUGIN_NAME%.clap" "%%COMMONPROGRAMFILES%%\CLAP\"
echo.

echo Or for per-user install:
echo   copy "%BUNDLE_DIR%\%PLUGIN_NAME%.clap" "%%LOCALAPPDATA%%\Programs\Common\CLAP\"

echo.

endlocal
