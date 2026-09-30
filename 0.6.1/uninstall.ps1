$ErrorActionPreference = "Stop"

$installDir = Join-Path $env:LOCALAPPDATA "SISR"
$appDataDir = Join-Path $env:APPDATA "SISR"

Write-Host "================================================"
Write-Host "SISR Uninstaller"
Write-Host "================================================"
Write-Host ""

if (-not (Test-Path $installDir)) {
    Write-Host "SISR does not appear to be installed at $installDir" -ForegroundColor Yellow
    exit 0
}

$procs = Get-Process -Name "SISR" -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "Stopping running SISR instance(s)..." -ForegroundColor Yellow
    $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

Write-Host "Removing SISR installation from $installDir..."
Remove-Item -Recurse -Force $installDir -ErrorAction SilentlyContinue
Write-Host "Removed installation directory" -ForegroundColor Green

if (Test-Path $appDataDir) {
    Write-Host "Removing SISR data/config from $appDataDir..."
    Remove-Item -Recurse -Force $appDataDir -ErrorAction SilentlyContinue
    Write-Host "Removed data directory" -ForegroundColor Green
}

$desktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "SISR.lnk"
if (Test-Path $desktopShortcut) {
    Remove-Item $desktopShortcut -Force -ErrorAction SilentlyContinue
    Write-Host "Removed desktop shortcut" -ForegroundColor Green
}

$startMenuShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\SISR.lnk"
if (Test-Path $startMenuShortcut) {
    Remove-Item $startMenuShortcut -Force -ErrorAction SilentlyContinue
    Write-Host "Removed Start Menu shortcut" -ForegroundColor Green
}

$startMenuNoSteamShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\SISR (No Steam).lnk"
if (Test-Path $startMenuNoSteamShortcut) {
    Remove-Item $startMenuNoSteamShortcut -Force -ErrorAction SilentlyContinue
    Write-Host "Removed Start Menu shortcut (No Steam)" -ForegroundColor Green
}

Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\SISR" -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "SISR has been uninstalled." -ForegroundColor Green
Write-Host ""

$viiperDir = Join-Path $env:LOCALAPPDATA "VIIPER"
$viiperExe = Join-Path $viiperDir "viiper.exe"

if (Test-Path $viiperExe) {
    $answer = Read-Host "Do you also want to uninstall VIIPER? [y/N]"
    if ($answer -match "^[Yy]") {
        Write-Host "Uninstalling VIIPER..."
        try {
            Start-Process -FilePath $viiperExe -ArgumentList "uninstall" -Wait -WindowStyle Hidden -ErrorAction SilentlyContinue
        }
        catch {
            Write-Host "Warning: viiper uninstall command failed (may already be stopped)" -ForegroundColor Yellow
        }
        $viiperProcs = Get-Process -Name "viiper" -ErrorAction SilentlyContinue
        if ($viiperProcs) {
            $viiperProcs | Stop-Process -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 500
        }
        Remove-Item -Recurse -Force $viiperDir -ErrorAction SilentlyContinue
        Write-Host "VIIPER uninstalled" -ForegroundColor Green
    }
    else {
        Write-Host "Skipping VIIPER uninstall" -ForegroundColor Yellow
    }
}

Write-Host ""

$usbipEntry = Get-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
    Where-Object { $_.DisplayName -like 'USBip version*' } |
    Select-Object -First 1
if (-not $usbipEntry) {
    $usbipEntry = Get-ItemProperty "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
        Where-Object { $_.DisplayName -like 'USBip version*' } |
        Select-Object -First 1
}

if ($usbipEntry) {
    $answer = Read-Host "Do you also want to uninstall USBIP (usbip-win2)? [y/N]"
    if ($answer -match "^[Yy]") {
        Write-Host "Uninstalling USBIP..."
        try {
            $uninstallString = $usbipEntry.UninstallString
            if ($uninstallString) {
                Start-Process -FilePath $uninstallString -Verb RunAs -Wait
                Write-Host "USBIP uninstalled" -ForegroundColor Green
                Write-Host "A reboot may be required to fully remove the USBIP drivers." -ForegroundColor Yellow
            }
            else {
                Write-Host "Warning: Could not find USBIP uninstall string in registry" -ForegroundColor Yellow
            }
        }
        catch {
            Write-Host "Warning: USBIP uninstall failed - $($_.Exception.Message)" -ForegroundColor Yellow
            Write-Host "You may need to uninstall usbip-win2 manually via Add/Remove Programs" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "Skipping USBIP uninstall" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Green
Write-Host "Uninstall complete." -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green