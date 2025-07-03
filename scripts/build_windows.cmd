@echo off
REM this_file: scripts/build_windows.cmd

setlocal enabledelayedexpansion

echo Building svgn for Windows...

REM Setup paths
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
set DIST_DIR=%PROJECT_ROOT%\dist\windows
set CARGO_DIR=%PROJECT_ROOT%\svgn

REM Clean and create dist directory
echo Creating distribution directory...
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: cargo not found. Please install Rust.
    exit /b 1
)

REM Determine architecture
set ARCH=%PROCESSOR_ARCHITECTURE%
if "%ARCH%"=="AMD64" (
    set TARGET=x86_64-pc-windows-msvc
    set ARCH_NAME=x64
) else if "%ARCH%"=="x86" (
    set TARGET=i686-pc-windows-msvc
    set ARCH_NAME=x86
) else if "%ARCH%"=="ARM64" (
    set TARGET=aarch64-pc-windows-msvc
    set ARCH_NAME=arm64
) else (
    echo Unsupported architecture: %ARCH%
    exit /b 1
)

echo Building for %ARCH_NAME% (%TARGET%)...

REM Add target if not already added
rustup target add %TARGET% 2>nul

REM Build release binary
cd "%CARGO_DIR%"
cargo build --release --target %TARGET%
if %ERRORLEVEL% neq 0 (
    echo Build failed!
    exit /b 1
)

REM Copy executable to dist directory
copy "target\%TARGET%\release\svgn.exe" "%DIST_DIR%\" >nul
if %ERRORLEVEL% neq 0 (
    echo Failed to copy executable!
    exit /b 1
)

REM Test the binary
echo Testing binary...
"%DIST_DIR%\svgn.exe" --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: Binary test failed!
    exit /b 1
)

REM Check binary size
for %%F in ("%DIST_DIR%\svgn.exe") do set SIZE=%%~zF
set /a SIZE_MB=%SIZE% / 1048576
if %SIZE_MB% gtr 50 (
    echo Warning: Binary size is %SIZE_MB%MB, consider optimization
)

REM Get version from Cargo.toml
for /f "tokens=2 delims==" %%i in ('findstr /r "^version" "%CARGO_DIR%\Cargo.toml"') do (
    set VERSION_LINE=%%i
    goto :version_found
)
:version_found
REM Remove quotes and spaces from version
set VERSION=%VERSION_LINE:"=%
set VERSION=%VERSION: =%
echo Building version: %VERSION%

REM Create zip archive
echo Creating zip archive...
cd "%DIST_DIR%"
powershell -Command "Compress-Archive -Path 'svgn.exe' -DestinationPath 'svgn-%VERSION%-windows-%ARCH_NAME%.zip' -Force"
cd "%PROJECT_ROOT%"

REM Create MSI installer if WiX is available
where candle >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo Creating MSI installer...
    
    REM Create WiX source file
    echo ^<?xml version="1.0" encoding="UTF-8"?^> > "%DIST_DIR%\svgn.wxs"
    echo ^<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi"^> >> "%DIST_DIR%\svgn.wxs"
    echo   ^<Product Id="*" Name="svgn" Language="1033" Version="%VERSION%.0" >> "%DIST_DIR%\svgn.wxs"
    echo            Manufacturer="svgn developers" UpgradeCode="a7c2e4b5-8d6f-4a3b-9c1e-2f5d7b9a4c6e"^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" /^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<MajorUpgrade DowngradeErrorMessage="A newer version is already installed." /^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<MediaTemplate EmbedCab="yes" /^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<Feature Id="ProductFeature" Title="svgn" Level="1"^> >> "%DIST_DIR%\svgn.wxs"
    echo       ^<ComponentGroupRef Id="ProductComponents" /^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^</Feature^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<Directory Id="TARGETDIR" Name="SourceDir"^> >> "%DIST_DIR%\svgn.wxs"
    echo       ^<Directory Id="ProgramFilesFolder"^> >> "%DIST_DIR%\svgn.wxs"
    echo         ^<Directory Id="INSTALLFOLDER" Name="svgn" /^> >> "%DIST_DIR%\svgn.wxs"
    echo       ^</Directory^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^</Directory^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^<ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER"^> >> "%DIST_DIR%\svgn.wxs"
    echo       ^<Component Id="svgn.exe" Guid="b8c3d5a7-2f6e-4d9c-8a1b-3e5f7c9d2b4a"^> >> "%DIST_DIR%\svgn.wxs"
    echo         ^<File Id="svgn.exe" Source="svgn.exe" KeyPath="yes"/^> >> "%DIST_DIR%\svgn.wxs"
    echo         ^<Environment Id="PATH" Name="PATH" Value="[INSTALLFOLDER]" >> "%DIST_DIR%\svgn.wxs"
    echo                      Permanent="no" Part="last" Action="set" System="yes" /^> >> "%DIST_DIR%\svgn.wxs"
    echo       ^</Component^> >> "%DIST_DIR%\svgn.wxs"
    echo     ^</ComponentGroup^> >> "%DIST_DIR%\svgn.wxs"
    echo   ^</Product^> >> "%DIST_DIR%\svgn.wxs"
    echo ^</Wix^> >> "%DIST_DIR%\svgn.wxs"
    
    REM Compile and link the MSI
    cd "%DIST_DIR%"
    candle svgn.wxs -o svgn.wixobj
    light svgn.wixobj -o "svgn-%VERSION%-windows-%ARCH_NAME%.msi"
    
    REM Clean up WiX files
    del svgn.wxs svgn.wixobj 2>nul
    cd "%PROJECT_ROOT%"
) else (
    echo WiX toolset not found, skipping MSI creation
)

REM Create installer using Inno Setup if available
set INNO_PATH=
for %%p in (
    "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
    "%ProgramFiles%\Inno Setup 6\ISCC.exe"
    "%ProgramFiles(x86)%\Inno Setup 5\ISCC.exe"
    "%ProgramFiles%\Inno Setup 5\ISCC.exe"
) do (
    if exist "%%~p" (
        set INNO_PATH=%%~p
        goto :inno_found
    )
)
:inno_found

if defined INNO_PATH (
    echo Creating Inno Setup installer...
    
    REM Create Inno Setup script
    echo [Setup] > "%DIST_DIR%\svgn.iss"
    echo AppName=svgn >> "%DIST_DIR%\svgn.iss"
    echo AppVersion=%VERSION% >> "%DIST_DIR%\svgn.iss"
    echo DefaultDirName={autopf}\svgn >> "%DIST_DIR%\svgn.iss"
    echo DefaultGroupName=svgn >> "%DIST_DIR%\svgn.iss"
    echo OutputDir=. >> "%DIST_DIR%\svgn.iss"
    echo OutputBaseFilename=svgn-%VERSION%-windows-%ARCH_NAME%-setup >> "%DIST_DIR%\svgn.iss"
    echo Compression=lzma2 >> "%DIST_DIR%\svgn.iss"
    echo SolidCompression=yes >> "%DIST_DIR%\svgn.iss"
    echo ChangesEnvironment=yes >> "%DIST_DIR%\svgn.iss"
    echo. >> "%DIST_DIR%\svgn.iss"
    echo [Files] >> "%DIST_DIR%\svgn.iss"
    echo Source: "svgn.exe"; DestDir: "{app}"; Flags: ignoreversion >> "%DIST_DIR%\svgn.iss"
    echo. >> "%DIST_DIR%\svgn.iss"
    echo [Registry] >> "%DIST_DIR%\svgn.iss"
    echo Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath('{app}') >> "%DIST_DIR%\svgn.iss"
    echo. >> "%DIST_DIR%\svgn.iss"
    echo [Code] >> "%DIST_DIR%\svgn.iss"
    echo function NeedsAddPath(Param: string): boolean; >> "%DIST_DIR%\svgn.iss"
    echo var >> "%DIST_DIR%\svgn.iss"
    echo   OrigPath: string; >> "%DIST_DIR%\svgn.iss"
    echo begin >> "%DIST_DIR%\svgn.iss"
    echo   if not RegQueryStringValue(HKEY_LOCAL_MACHINE, >> "%DIST_DIR%\svgn.iss"
    echo     'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', >> "%DIST_DIR%\svgn.iss"
    echo     'Path', OrigPath) then >> "%DIST_DIR%\svgn.iss"
    echo   begin >> "%DIST_DIR%\svgn.iss"
    echo     Result := True; >> "%DIST_DIR%\svgn.iss"
    echo     exit; >> "%DIST_DIR%\svgn.iss"
    echo   end; >> "%DIST_DIR%\svgn.iss"
    echo   Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0; >> "%DIST_DIR%\svgn.iss"
    echo end; >> "%DIST_DIR%\svgn.iss"
    
    REM Compile the installer
    cd "%DIST_DIR%"
    "%INNO_PATH%" svgn.iss
    
    REM Clean up
    del svgn.iss 2>nul
    cd "%PROJECT_ROOT%"
) else (
    echo Inno Setup not found, skipping installer creation
)

REM Create a batch install script
echo @echo off > "%DIST_DIR%\install.bat"
echo echo Installing svgn... >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo set INSTALL_DIR=%%ProgramFiles%%\svgn >> "%DIST_DIR%\install.bat"
echo if not exist "%%INSTALL_DIR%%" mkdir "%%INSTALL_DIR%%" >> "%DIST_DIR%\install.bat"
echo copy /Y svgn.exe "%%INSTALL_DIR%%\" >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo echo Adding to PATH... >> "%DIST_DIR%\install.bat"
echo setx PATH "%%PATH%%;%%INSTALL_DIR%%" /M >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo echo Installation complete! >> "%DIST_DIR%\install.bat"
echo echo Please restart your command prompt and run 'svgn --help' >> "%DIST_DIR%\install.bat"
echo pause >> "%DIST_DIR%\install.bat"

REM Summary
echo.
echo Build complete!
echo Distribution files created in: %DIST_DIR%
echo   - svgn.exe
echo   - svgn-%VERSION%-windows-%ARCH_NAME%.zip
if exist "%DIST_DIR%\svgn-%VERSION%-windows-%ARCH_NAME%.msi" (
    echo   - svgn-%VERSION%-windows-%ARCH_NAME%.msi
)
if exist "%DIST_DIR%\svgn-%VERSION%-windows-%ARCH_NAME%-setup.exe" (
    echo   - svgn-%VERSION%-windows-%ARCH_NAME%-setup.exe
)
echo   - install.bat

endlocal