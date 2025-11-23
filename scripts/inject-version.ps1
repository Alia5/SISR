param(
    [string]$Version,
    [string]$InputJson,
    [string]$OutputJson
)

if (-not $Version) { $Version = '0.0.0-dev' }
$ver = $Version.TrimStart('v')
$parts = $ver -split '[.-]'

$major = 0; $minor = 0; $patch = 0; $build = 0

if ($parts.Length -ge 1 -and ($parts[0] -match '^\d+$')) {
    try { $major = [int]$parts[0] } catch { $major = 0 }
    if ($parts.Length -ge 2 -and ($parts[1] -match '^\d+$')) { try { $minor = [int]$parts[1] } catch { $minor = 0 } }
    if ($parts.Length -ge 3 -and ($parts[2] -match '^\d+$')) { try { $patch = [int]$parts[2] } catch { $patch = 0 } }
    if ($parts.Length -ge 4) {
        $buildStr = $parts[3]
        if ($buildStr -match '^\d+$') { try { $build = [int]$buildStr } catch { $build = 0 } }
    }
} else {
    $ver = '0.0.0-dev'
}

if (-not (Test-Path $InputJson)) { Write-Error "Missing input JSON: $InputJson"; exit 1 }
$json = Get-Content $InputJson -Raw | ConvertFrom-Json

$json.FixedFileInfo.FileVersion.Major = $major
$json.FixedFileInfo.FileVersion.Minor = $minor
$json.FixedFileInfo.FileVersion.Patch = $patch
$json.FixedFileInfo.FileVersion.Build = $build
$json.FixedFileInfo.ProductVersion.Major = $major
$json.FixedFileInfo.ProductVersion.Minor = $minor
$json.FixedFileInfo.ProductVersion.Patch = $patch
$json.FixedFileInfo.ProductVersion.Build = $build
$json.StringFileInfo.FileVersion = "$major.$minor.$patch.$build"
$json.StringFileInfo.ProductVersion = $ver

$json | ConvertTo-Json -Depth 10 | Set-Content $OutputJson
Write-Host "Version injection complete: FileVersion=$major.$minor.$patch.$build ProductVersion=$ver"
