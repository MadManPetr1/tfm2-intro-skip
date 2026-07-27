param()

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Require-File([string]$relativePath) {
    $path = Join-Path $root $relativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required file is missing: $relativePath"
    }
}

@(
    "Cargo.toml",
    "Cargo.lock",
    "src/lib.rs",
    "mod.mod_info",
    "mod.override_info",
    "settings.json",
    "thumbnail.png",
    "assets/banner.png",
    "assets/logo-1024.png",
    "assets/logo-512.png",
    "assets/logo-256.png",
    "assets/logo-128.png",
    "assets/logo-64.png",
    "assets/logo-32.png",
    "ui/layout/title.ui",
    "README.md",
    "LICENSE"
) | ForEach-Object { Require-File $_ }

function Get-PngDimensions([string]$relativePath) {
    $path = Join-Path $root $relativePath
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $signature = @(137, 80, 78, 71, 13, 10, 26, 10)

    if ($bytes.Length -lt 24) {
        throw "PNG is too short: $relativePath"
    }
    for ($index = 0; $index -lt $signature.Length; $index++) {
        if ($bytes[$index] -ne $signature[$index]) {
            throw "Invalid PNG signature: $relativePath"
        }
    }

    $width = ([uint32]$bytes[16] * 16777216) +
        ([uint32]$bytes[17] * 65536) +
        ([uint32]$bytes[18] * 256) +
        [uint32]$bytes[19]
    $height = ([uint32]$bytes[20] * 16777216) +
        ([uint32]$bytes[21] * 65536) +
        ([uint32]$bytes[22] * 256) +
        [uint32]$bytes[23]

    return @($width, $height)
}

$expectedPngDimensions = @{
    "thumbnail.png" = @(512, 512)
    "assets/banner.png" = @(1280, 640)
    "assets/logo-1024.png" = @(1024, 1024)
    "assets/logo-512.png" = @(512, 512)
    "assets/logo-256.png" = @(256, 256)
    "assets/logo-128.png" = @(128, 128)
    "assets/logo-64.png" = @(64, 64)
    "assets/logo-32.png" = @(32, 32)
}

foreach ($relativePath in $expectedPngDimensions.Keys) {
    $actual = Get-PngDimensions $relativePath
    $expected = $expectedPngDimensions[$relativePath]
    if ($actual[0] -ne $expected[0] -or $actual[1] -ne $expected[1]) {
        throw "$relativePath must be $($expected[0])x$($expected[1]); found $($actual[0])x$($actual[1])."
    }
}

$modInfo = Get-Content -LiteralPath (Join-Path $root "mod.mod_info") -Raw | ConvertFrom-Json
$settings = Get-Content -LiteralPath (Join-Path $root "settings.json") -Raw | ConvertFrom-Json
$override = Get-Content -LiteralPath (Join-Path $root "mod.override_info") -Raw | ConvertFrom-Json
$cargo = Get-Content -LiteralPath (Join-Path $root "Cargo.toml") -Raw

$cargoVersionMatch = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
    throw "Could not read the package version from Cargo.toml."
}
if ($cargoVersionMatch.Groups[1].Value -ne $modInfo.version) {
    throw "Version mismatch: Cargo.toml is $($cargoVersionMatch.Groups[1].Value), mod.mod_info is $($modInfo.version)."
}

foreach ($setting in "skip_disclaimer", "auto_continue", "auto_load_anyway") {
    if ($null -eq $settings.$setting -or $settings.$setting -isnot [bool]) {
        throw "settings.json must contain a Boolean '$setting' value."
    }
}

if ($override.PSObject.Properties.Name -notcontains "asset/base/ui/layout/title") {
    throw "mod.override_info does not remap the title layout."
}

$source = Get-Content -LiteralPath (Join-Path $root "src/lib.rs") -Raw
if ($source -notmatch 'const MOD_ID: &str = "intro_skip";') {
    throw 'The Rust MOD_ID must remain "intro_skip".'
}
if ($source -notmatch 'const MOD_NAME: &str = "Intro Skip";') {
    throw 'The Rust MOD_NAME must match mod.mod_info.'
}

Push-Location $root
try {
    cargo fmt --check
    if ($LASTEXITCODE -ne 0) {
        throw "cargo fmt --check failed."
    }
}
finally {
    Pop-Location
}

Write-Host "Repository validation passed for Intro Skip v$($modInfo.version)."
