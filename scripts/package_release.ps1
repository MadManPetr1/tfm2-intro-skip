# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

param(
    [string]$SdkDir = $env:TFM2_MOD_SDK,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

& (Join-Path $PSScriptRoot "validate_repo.ps1")

if (-not $SkipBuild) {
    & (Join-Path $root "build_local.ps1") -SdkDir $SdkDir
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with exit code $LASTEXITCODE."
    }
}

$dll = Join-Path $root "tfm2_intro_skip.dll"
if (-not (Test-Path -LiteralPath $dll -PathType Leaf)) {
    throw "tfm2_intro_skip.dll is missing. Build first or remove -SkipBuild."
}

$modInfo = Get-Content -LiteralPath (Join-Path $root "mod.mod_info") -Raw | ConvertFrom-Json
$buildRoot = Join-Path $root "builds"
$releaseRoot = Join-Path $buildRoot "tfm2-intro-skip-v$($modInfo.version)"
$runtimeRoot = Join-Path $releaseRoot "tfm2_intro_skip"
$archive = Join-Path $buildRoot "tfm2-intro-skip-v$($modInfo.version).zip"

if (Test-Path -LiteralPath $releaseRoot) {
    Remove-Item -LiteralPath $releaseRoot -Recurse -Force
}
if (Test-Path -LiteralPath $archive) {
    Remove-Item -LiteralPath $archive -Force
}

New-Item -ItemType Directory -Path (Join-Path $runtimeRoot "ui\layout") -Force | Out-Null

@(
    "tfm2_intro_skip.dll",
    "mod.mod_info",
    "mod.override_info",
    "better_mod_menu.json",
    "better_mod_menu_profile.json",
    "profile_icon.png",
    "banner.png",
    "thumbnail.png",
    "README.md",
    "CHANGELOG.md",
    "LICENSE",
    "NOTICE.md"
) | ForEach-Object {
    Copy-Item -LiteralPath (Join-Path $root $_) -Destination (Join-Path $runtimeRoot $_)
}

Copy-Item `
    -LiteralPath (Join-Path $root "ui\layout\title.ui") `
    -Destination (Join-Path $runtimeRoot "ui\layout\title.ui")

Compress-Archive -LiteralPath $runtimeRoot -DestinationPath $archive -CompressionLevel Optimal

Write-Host "Release package created: $archive"
Write-Host "Runtime folder created: $runtimeRoot"
