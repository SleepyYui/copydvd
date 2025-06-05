# PowerShell script to create a Windows installer for Copy DVD
param(
    [string]$Target = "x86_64-pc-windows-msvc"
)

$AppName = "Copy DVD"
# Extract version from Cargo.toml
$Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$BinaryPath = "target\$Target\release\copydvd.exe"

Write-Host "Creating Windows installer for $AppName..."

# Check if binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath"
    exit 1
}

# Create installer directory structure
$InstallerDir = "installer-windows"
$AppDir = "$InstallerDir\$AppName"

if (Test-Path $InstallerDir) {
    Remove-Item -Recurse -Force $InstallerDir
}

New-Item -ItemType Directory -Path $AppDir -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir\resources" -Force | Out-Null

# Copy binary and resources
Copy-Item $BinaryPath "$AppDir\copydvd.exe"
if (Test-Path "resources") {
    Copy-Item -Recurse "resources\*" "$AppDir\resources\"
}

# Create uninstaller script
$UninstallScript = @"
@echo off
echo Uninstalling $AppName...
rmdir /s /q "%PROGRAMFILES%\$AppName"
echo $AppName has been uninstalled.
pause
"@

$UninstallScript | Out-File -FilePath "$AppDir\uninstall.bat" -Encoding ASCII

# Create simple installer script
$InstallScript = @"
@echo off
echo Installing $AppName...
mkdir "%PROGRAMFILES%\$AppName" 2>nul
xcopy /e /i /y "%~dp0\*" "%PROGRAMFILES%\$AppName\"
echo $AppName has been installed to: %PROGRAMFILES%\$AppName
echo.
echo You can now run it from: %PROGRAMFILES%\$AppName\copydvd.exe
pause
"@

$InstallScript | Out-File -FilePath "$InstallerDir\install.bat" -Encoding ASCII

# Create README for the installer
$ReadmeContent = @"
# $AppName Installer

This is a simple installer for $AppName v$Version.

## Installation
1. Run 'install.bat' as Administrator
2. The application will be installed to: C:\Program Files\$AppName\

## Usage
- Run the GUI: copydvd.exe
- Run the CLI: copydvd.exe --cli

## Uninstallation
Run the uninstall.bat file from the installation directory.

"@

$ReadmeContent | Out-File -FilePath "$InstallerDir\README.txt" -Encoding UTF8

Write-Host "Windows installer created in: $InstallerDir"
Write-Host "To install, run: $InstallerDir\install.bat (as Administrator)"