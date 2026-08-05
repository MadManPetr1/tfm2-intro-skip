# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
    "better_mod_menu.json",
    "better_mod_menu_profile.json",
    "profile_icon.png",
    "banner.png",
    "settings.json",
    "thumbnail.png",
    "assets/thumbnail-master.png",
    "ui/layout/title.ui",
    "README.md",
    "workshop_description.txt",
    "CHANGELOG.md",
    "LICENSE",
    "NOTICE.md",
    "scripts/package_release.ps1"
) | ForEach-Object { Require-File $_ }

$profile = Get-Content -LiteralPath (Join-Path $root "better_mod_menu_profile.json") -Raw |
    ConvertFrom-Json
if ($profile.schema_version -ne 1 -or $profile.profile_icon -ne "profile_icon.png") {
    throw "better_mod_menu_profile.json must use schema version 1 and profile_icon.png."
}
$bmmManifest = Get-Content -LiteralPath (Join-Path $root "better_mod_menu.json") -Raw |
    ConvertFrom-Json
if ($bmmManifest.schema_version -ne 1 -or $bmmManifest.mod_id -ne "tfm2_intro_skip") {
    throw "better_mod_menu.json must target Intro Skip with schema version 1."
}

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
    "thumbnail.png" = @(256, 256)
    "assets/thumbnail-master.png" = @(128, 128)
    "banner.png" = @(1280, 640)
}

foreach ($relativePath in $expectedPngDimensions.Keys) {
    $actual = Get-PngDimensions $relativePath
    $expected = $expectedPngDimensions[$relativePath]
    if ($actual[0] -ne $expected[0] -or $actual[1] -ne $expected[1]) {
        throw "$relativePath must be $($expected[0])x$($expected[1]); found $($actual[0])x$($actual[1])."
    }
}

$modInfo = Get-Content -LiteralPath (Join-Path $root "mod.mod_info") -Raw | ConvertFrom-Json
if ($modInfo.mod_id -ne "tfm2_intro_skip") {
    throw "mod.mod_info must declare mod_id tfm2_intro_skip."
}
$settings = Get-Content -LiteralPath (Join-Path $root "settings.json") -Raw | ConvertFrom-Json
$override = Get-Content -LiteralPath (Join-Path $root "mod.override_info") -Raw | ConvertFrom-Json
$cargo = Get-Content -LiteralPath (Join-Path $root "Cargo.toml") -Raw
$cargoLock = Get-Content -LiteralPath (Join-Path $root "Cargo.lock") -Raw
$workshop = Get-Content -LiteralPath (Join-Path $root "workshop_description.txt") -Raw

$cargoVersionMatch = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
    throw "Could not read the package version from Cargo.toml."
}
$lockVersion = [regex]::Match(
    $cargoLock,
    '(?ms)\[\[package\]\]\s+name\s*=\s*"tfm2_intro_skip"\s+version\s*=\s*"([^"]+)"'
).Groups[1].Value
if ($cargoVersionMatch.Groups[1].Value -ne $modInfo.version -or
    $lockVersion -ne $modInfo.version) {
    throw "Version mismatch between Cargo.toml, Cargo.lock, and mod.mod_info."
}
if ($cargo -notmatch '(?m)^license\s*=\s*"MPL-2\.0"') {
    throw "Cargo.toml must declare MPL-2.0."
}
$base = @($modInfo.dependencies | Where-Object { $_.mod_id -eq "base" })
if ($base.Count -ne 1 -or $base[0].version -ne ">=0.5.2, <0.5.5") {
    throw "Intro Skip must declare the supported 0.5.2-0.5.4 base range."
}
if ($bmmManifest.display.version -ne $modInfo.version) {
    throw "better_mod_menu.json version must match mod.mod_info."
}
foreach ($expected in @(
    "[code]tfm2_intro_skip.dll[/code]",
    "[b]Current version:[/b] v$($modInfo.version)",
    "[url=https://github.com/MadManPetr1/tfm2-intro-skip]Source code on GitHub[/url]"
)) {
    if ($workshop -notmatch [regex]::Escape($expected)) {
        throw "Workshop description is missing or inconsistent: $expected"
    }
}
if ($workshop -notmatch '\[b\]Tested with:\[/b\] TFM2 0\.5\.2.+0\.5\.4') {
    throw "Workshop Tested with line must match the supported base range."
}
if ($workshop -notmatch '(?m)^\[b\]Last tested:\[/b\] \d{2}/\d{2}/\d{4}\r?$') {
    throw "Workshop Last tested must use DD/MM/YYYY."
}

foreach ($setting in "skip_disclaimer", "auto_continue", "auto_load_anyway") {
    if ($null -eq $settings.$setting -or $settings.$setting -isnot [bool]) {
        throw "settings.json must contain a Boolean '$setting' value."
    }
}
if ($settings.backup_retention_days -notin @(3, 7, 14, 30)) {
    throw "settings.json backup_retention_days must be 3, 7, 14, or 30."
}

if ($override.PSObject.Properties.Name -notcontains "asset/base/ui/layout/title") {
    throw "mod.override_info does not remap the title layout."
}

$source = Get-Content -LiteralPath (Join-Path $root "src/lib.rs") -Raw
if ($source -notmatch 'const MOD_ID: &str = "tfm2_intro_skip";') {
    throw 'The Rust MOD_ID must remain "tfm2_intro_skip".'
}
if ($source -notmatch 'const LEGACY_MOD_ID: &str = "intro_skip";') {
    throw 'The legacy Intro Skip identity must remain available for settings migration.'
}
if ($source -notmatch 'const MOD_NAME: &str = "Intro Skip";') {
    throw 'The Rust MOD_NAME must match mod.mod_info.'
}

$titleLayout = Get-Content -LiteralPath (Join-Path $root "ui/layout/title.ui") -Raw
$headerVersion = [regex]::Escape("text: `"Version $($modInfo.version)`";")
if ($titleLayout -notmatch $headerVersion) {
    throw "The Intro Skip header version must match mod.mod_info."
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
