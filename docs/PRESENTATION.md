# Public presentation copy

This page contains ready-to-paste text for the repository and Steam Workshop launch.

## GitHub repository

**Repository name**

`tfm2-intro-skip`

**Description**

Safe, configurable startup automation for Teamfight Manager 2—with verified save backups before automatic mod-mismatch loading.

**Website**

https://steamcommunity.com/sharedfiles/filedetails/?id=3773405658

**Topics**

`teamfight-manager-2`, `teamfight-manager2`, `mod`, `rust`, `quality-of-life`, `save-backup`, `steam-workshop`

**Social preview**

Upload `assets/banner.png` in **Settings → General → Social preview**.

**Logo assets**

- `thumbnail.png` — 512×512 Workshop/mod thumbnail.
- `assets/logo-1024.png` — editable master export.
- `assets/logo-512.png`, `logo-256.png`, `logo-128.png`, `logo-64.png`, and `logo-32.png` — nearest-neighbour exports for profile and announcement use.

## Steam Workshop

**Title**

Intro Skip — Safe Startup

**Short description**

Skip the disclaimer, optionally continue your latest career, and safely automate mod-mismatch loading only after a verified backup.

**Full description**

Intro Skip removes repetitive startup clicks while keeping the risky parts under your control.

FEATURES

• Skip the startup disclaimer  
• Optionally select Continue automatically  
• Optionally select Load Anyway on a mod mismatch  
• Create and verify a backup before automatic mismatch loading  
• Keep managed backups for 7 days  
• Import the latest backup into the normal Load menu manually  
• Configure everything from the in-game Mods screen

SAFE BY DEFAULT

Only disclaimer skipping is enabled in the published configuration. Automatic Continue and automatic Load Anyway are opt-in.

If automatic Load Anyway is enabled, Intro Skip first copies the latest career save and verifies that the copied bytes match. If the backup fails, the automatic load is stopped.

COMPATIBILITY

Intro Skip 0.4.7 supports Teamfight Manager 2 0.5.2 and 0.5.3.

RECOVERY

Automatic backups are retained for 3, 7, 14, or 30 days. Select one of the five latest verified backups in the Mods screen to create a separate recovery save. Recovery is never loaded automatically.

This is an independent community mod and is not affiliated with or endorsed by Team Samoyed.

SOURCE AND CONTRIBUTIONS

Source code and documentation are available under MPL-2.0. Distributed
modifications to covered files must remain available under the same license.
The original logo, banner, and thumbnail artwork are All Rights Reserved.
Forks must use their own name and artwork.

## GitHub v0.4.7 release

**Release title**

Intro Skip v0.4.7 — TFM2 0.5.2–0.5.3 Compatibility

**Release notes**

Intro Skip provides safe, configurable startup automation for Teamfight Manager 2 0.5.2 and 0.5.3.

Version 0.4.7 rebuilds the native mod against the 0.5.2 compatibility baseline
and verifies that exact DLL on 0.5.3, extending support without separate
per-game-version packages.

Highlights:

- skip the startup disclaimer;
- optionally select Continue;
- optionally accept a mod-state mismatch;
- create and byte-verify a backup before automatic Load Anyway;
- retain managed backups for 3, 7, 14, or 30 days;
- import any of the five latest verified backups into the normal Load menu manually;
- configure all behavior from the in-game Mods screen;
- use the same controls through Better Mod Menu when it is installed.

Only disclaimer skipping is enabled by default. Automatic Continue and Load Anyway remain opt-in.

Install `intro-skip-v0.4.7.zip` from the assets below. Do not use GitHub's automatic Source code archives for installation.

## Launch post

**Intro Skip for Teamfight Manager 2 is now available.**

It skips the disclaimer, can continue your latest career automatically, and can handle mod-state mismatch prompts—but only after creating and verifying a save backup. The riskier options are off by default, backups are retained for seven days, and recovery always stays manual.

Supports TFM2 0.5.2 and 0.5.3.

## Screenshot plan

Use real in-game captures; do not mock these.

1. **Hero:** title screen with the Mods window and Intro Skip selected.
2. **Settings:** tight crop of the full Intro Skip settings panel.
3. **Safety:** mod-mismatch popup immediately before the guarded automatic action.
4. **Recovery:** success state after importing a backup into the Load menu.
5. **Proof:** Load menu showing the separately imported recovery save, with personal save names blurred if necessary.

Recommended order: hero, settings, safety, recovery, proof.
