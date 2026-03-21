$ErrorActionPreference = "Stop"

$sisrVersion = "dev-snapshot"
$viiperVersion = "dev-snapshot"

$repo = "Alia5/SISR"
$apiUrl = "https://api.github.com/repos/$repo/releases/latest"

if ($sisrVersion -eq "dev-snapshot") {
    $apiUrl = "https://api.github.com/repos/$repo/releases/tags/dev-snapshot"
}
elseif ($sisrVersion -match "^v?\d+\.\d+") {
    $apiUrl = "https://api.github.com/repos/$repo/releases/tags/$sisrVersion"
}

Write-Host "Fetching SISR release: $sisrVersion..." -ForegroundColor Cyan
$releaseData = Invoke-RestMethod -Uri $apiUrl -ErrorAction Stop
$version = $releaseData.tag_name

if (-not $version) {
    Write-Host "Error: Could not fetch SISR release" -ForegroundColor Red
    exit 1
}

Write-Host "Version: $version" -ForegroundColor Green

$arch = if ([Environment]::Is64BitOperatingSystem) {
    if ((Get-CimInstance Win32_ComputerSystem).SystemType -match "ARM") {
        "aarch64"
    }
    else {
        "x86_64"
    }
}
else {
    Write-Host "Error: Only 64-bit Windows is supported" -ForegroundColor Red
    exit 1
}

$buildType = if ($version -match "snapshot") { "Snapshot" } else { "Release" }
$assetName = "SISR-$arch-windows-msvc-$buildType.zip"

Write-Host "Architecture: $arch" -ForegroundColor Cyan
Write-Host "Looking for asset: $assetName" -ForegroundColor Cyan

$asset = $releaseData.assets | Where-Object { $_.name -eq $assetName }
if (-not $asset) {
    Write-Host "Error: Could not find asset $assetName" -ForegroundColor Red
    exit 1
}

$downloadUrl = $asset.browser_download_url
Write-Host "Downloading from: $downloadUrl" -ForegroundColor Cyan

$tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

try {
    $tempZip = Join-Path $tempDir "sisr.zip"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -ErrorAction Stop
    Write-Host "Downloaded successfully" -ForegroundColor Green
    
    $installDir = Join-Path $env:LOCALAPPDATA "SISR"
    $isUpdate = Test-Path $installDir
    
    Write-Host "Installing to $installDir..." -ForegroundColor Cyan
    
    if ($isUpdate) {
        Write-Host "Existing SISR installation detected" -ForegroundColor Yellow
        $procs = Get-Process -Name "SISR" -ErrorAction SilentlyContinue
        if ($procs) {
            Write-Host "Stopping running SISR instance(s)..." -ForegroundColor Yellow
            $procs | Stop-Process -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 1
        }
    }
    
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    Expand-Archive -Path $tempZip -DestinationPath $installDir -Force
    Write-Host "Extracted SISR to $installDir" -ForegroundColor Green
    
    Write-Host ""
    Write-Host "Installing VIIPER version: $viiperVersion" -ForegroundColor Cyan
    $viiperInstallVersion = $viiperVersion
    if ($viiperInstallVersion -eq "dev-snapshot") {
        $viiperInstallVersion = "main"
    }
    $viiperScript = Join-Path $tempDir "viiper-install.ps1"
    try {
        Invoke-WebRequest -Uri "https://alia5.github.io/VIIPER/$viiperInstallVersion/install.ps1" -OutFile $viiperScript -ErrorAction Stop
        & powershell -ExecutionPolicy Bypass -File $viiperScript
        Write-Host "VIIPER installed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Warning: VIIPER installation failed. You may need to install it manually." -ForegroundColor Yellow
        Write-Host "See: https://alia5.github.io/VIIPER/stable/getting-started/installation/" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "Configuring Steam CEF remote debugging..." -ForegroundColor Cyan
    
    $steamPaths = @()
    
    try {
        $steamPath = (Get-ItemProperty -Path "HKCU:\Software\Valve\Steam" -Name "SteamPath" -ErrorAction SilentlyContinue).SteamPath
        if ($steamPath) {
            $steamPaths += $steamPath
        }
    }
    catch {}
    
    $steamPaths += "C:\Program Files (x86)\Steam"
    $steamPaths += "C:\Program Files\Steam" # will maybe exist in the future?
    
    $cefCreated = $false
    foreach ($steamPath in $steamPaths) {
        if (Test-Path $steamPath) {
            $cefFile = Join-Path $steamPath ".cef-enable-remote-debugging"
            try {
                if (-not (Test-Path $cefFile)) {
                    New-Item -ItemType File -Path $cefFile -Force | Out-Null
                    Write-Host "Created CEF debug file in: $steamPath" -ForegroundColor Green
                    $cefCreated = $true
                }
                else {
                    Write-Host "CEF debug file already exists in: $steamPath" -ForegroundColor Green
                    $cefCreated = $true
                }
            }
            catch {
                Write-Host "Warning: Could not create CEF debug file in $steamPath" -ForegroundColor Yellow
            }
        }
    }
    
    if (-not $cefCreated) {
        Write-Host "Warning: Could not find Steam installation or create CEF debug file" -ForegroundColor Yellow
        Write-Host "You may need to manually create .cef-enable-remote-debugging in your Steam directory" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "Creating shortcuts..." -ForegroundColor Cyan
    
    $sisrExe = Join-Path $installDir "SISR.exe"
    $WshShell = New-Object -ComObject WScript.Shell
    
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $desktopShortcut = Join-Path $desktopPath "SISR.lnk"
    try {
        $shortcut = $WshShell.CreateShortcut($desktopShortcut)
        $shortcut.TargetPath = $sisrExe
        $shortcut.WorkingDirectory = $installDir
        $shortcut.Save()
        Write-Host "Created desktop shortcut" -ForegroundColor Green
    }
    catch {
        Write-Host "Warning: Could not create desktop shortcut - $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    $startMenuPath = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    $startMenuShortcut = Join-Path $startMenuPath "SISR.lnk"
    try {
        $shortcut = $WshShell.CreateShortcut($startMenuShortcut)
        $shortcut.TargetPath = $sisrExe
        $shortcut.WorkingDirectory = $installDir
        $shortcut.Save()
        Write-Host "Created Start Menu shortcut" -ForegroundColor Green
    }
    catch {
        Write-Host "Warning: Could not create Start Menu shortcut - $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "================================================" -ForegroundColor Green
    Write-Host "SISR installed successfully!" -ForegroundColor Green
    Write-Host "================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installation location: $installDir" -ForegroundColor Cyan
    Write-Host "Executable: $sisrExe" -ForegroundColor Cyan
    Write-Host "You can now run SISR from the Desktop or Start Menu shortcut." -ForegroundColor Green
    Write-Host "" 
    
    if ($isUpdate) {
        Write-Host "Update complete!" -ForegroundColor Green
    }
    
}
finally {
    Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
}
