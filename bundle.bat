@echo off
REM Bundle script for creating VST3/CLAP plugin bundles on Windows

setlocal enabledelayedexpansion

set PLUGIN_NAME=DSynth
set BUILD_DIR=target\release
set BUNDLE_DIR=target\bundled

echo Building DSynth plugin bundles for Windows...

REM Build the plugin
echo Building release binary...
cargo build --release --lib --features vst

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

REM Create bundle directories
echo Creating bundle structure...
if exist "%BUNDLE_DIR%" rmdir /s /q "%BUNDLE_DIR%"
mkdir "%BUNDLE_DIR%"

REM Create VST3 bundle
set VST3_BUNDLE=%BUNDLE_DIR%\%PLUGIN_NAME%.vst3
mkdir "%VST3_BUNDLE%\Contents\x86_64-win"

REM Copy and rename the dll
echo Copying plugin binary...
copy "%BUILD_DIR%\dsynth.dll" "%VST3_BUNDLE%\Contents\x86_64-win\%PLUGIN_NAME%.vst3"

REM Create moduleinfo.json for VST3
echo Creating moduleinfo.json...
(
echo {
echo   "Name": "DSynth",
echo   "Version": "0.1.1",
echo   "Factory Info": {
echo     "Vendor": "DSynth",
echo     "URL": "",
echo     "E-Mail": ""
echo   },
echo   "Compatibility": {
echo     "Classes": [
echo       {
echo         "CID": "44535374445300000000000000000000"
echo       }
echo     ]
echo   }
echo }
) > "%VST3_BUNDLE%\Contents\x86_64-win\moduleinfo.json"

REM Create CLAP bundle
set CLAP_BUNDLE=%BUNDLE_DIR%\%PLUGIN_NAME%.clap
mkdir "%CLAP_BUNDLE%"

REM Copy and rename the dll for CLAP
copy "%BUILD_DIR%\dsynth.dll" "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap"

echo.
echo ✅ Plugin bundles created successfully!
echo.
echo VST3: %VST3_BUNDLE%
echo CLAP: %CLAP_BUNDLE%
echo.
echo To install:
echo   xcopy /E /I "%VST3_BUNDLE%" "%%COMMONPROGRAMFILES%%\VST3\%PLUGIN_NAME%.vst3"
echo   copy "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap" "%%COMMONPROGRAMFILES%%\CLAP\"
echo.
echo Or for per-user install:
echo   xcopy /E /I "%VST3_BUNDLE%" "%%LOCALAPPDATA%%\Programs\Common\VST3\%PLUGIN_NAME%.vst3"
echo   copy "%CLAP_BUNDLE%\%PLUGIN_NAME%.clap" "%%LOCALAPPDATA%%\Programs\Common\CLAP\"
echo.

endlocal
