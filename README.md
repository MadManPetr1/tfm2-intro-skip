![Intro Skip banner](assets/banner.png)

<div align="center">

# Intro Skip

Safe, configurable startup automation for **Teamfight Manager 2**.

[Features](#features) · [Installation](#installation) · [Configuration](#configuration) · [Safety](#save-safety) · [Building](#building-from-source)

</div>

> [!IMPORTANT]
> Version **0.4.7** supports Teamfight Manager 2 **0.5.2 and 0.5.3**.

## Features

- Skips the startup disclaimer.
- Optionally selects **Continue** automatically.
- Optionally accepts the mod-state mismatch prompt.
- Creates and byte-verifies a save backup before automatic **Load Anyway**.
- Stops the automatic mismatch flow if backup creation or verification fails.
- Retains managed backups for a configurable 3, 7, 14, or 30 days.
- Keeps recovery manual: a backup is imported into the regular **Load** menu only when you request it.
- Provides all settings directly inside the game's Mods screen.
- Integrates with Better Mod Menu when installed while preserving the native
  settings panel as a standalone fallback.

| Setting | Included default |
| --- | --- |
| Skip disclaimer | On |
| Automatically select Continue | Off |
| Automatically Load Anyway on mod mismatch | Off |
| Backup retention | 7 days |

## Installation

### Steam Workshop

[Subscribe to Intro Skip](https://steamcommunity.com/sharedfiles/filedetails/?id=3773405658),
enable it in the in-game Mods menu, then restart the game when prompted.

### Manual GitHub release

1. Download `intro-skip-vX.Y.Z.zip` from this repository's **Releases** page. Do not download GitHub's automatic “Source code” archive.
2. Extract the included `intro_skip` folder into:

   ```text
   ...\SteamLibrary\steamapps\common\Teamfight Manager2\mods\
   ```

3. Confirm this exact structure:

   ```text
   Teamfight Manager2\mods\intro_skip\mod.mod_info
   Teamfight Manager2\mods\intro_skip\intro_skip.dll
   ```

4. Enable **Intro Skip** in the Mods menu and restart the game.

## Configuration

Open **Mods**, select **Intro Skip**, and use the settings panel:

- **Skip disclaimer** controls only the startup disclaimer.
- **Automatically select Continue** loads the most recent career after the title UI becomes ready.
- **Automatically Load Anyway on mod mismatch** is deliberately off by default. When enabled, it proceeds only after a verified backup exists.
- **Backup retention** keeps managed backups for 3, 7, 14, or 30 days.
- **Recent verified backups** lists up to five backups, newest first. Selecting one imports a separate recovery save into the regular Load menu.
- **Visible status** confirms saved settings, retention cleanup, successful imports, and failures.

Changes are written outside the Workshop-managed mod folder so updates cannot replace them:

```text
%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip\settings.json
```

Existing `mods\intro_skip\settings.json` values are migrated automatically on first launch.

When Better Mod Menu is enabled, the same controls and recent-backup actions
appear in its Settings tab. Intro Skip still validates and performs every
backup action itself.

## Save safety

Automatic mismatch acceptance follows a guarded sequence:

1. Find the latest `save_*.data` career save.
2. Copy it to the dedicated backup folder.
3. Verify both size and contents.
4. Only then activate **Load Anyway**.

If any step fails, the mod does not press **Load Anyway**.

Backups are stored in:

```text
%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip_backups
```

Managed backups older than the selected retention period are removed. Diagnostic messages are appended to:

```text
%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip\backup_status.log
```

> [!WARNING]
> No mod can guarantee save compatibility when a career's enabled mods differ. The backup guard reduces recovery risk; it does not make incompatible mod combinations safe.

## Requirements and limitations

- Teamfight Manager 2 `0.5.2` or `0.5.3`
- Windows
- The `0.5.2` Mod SDK compatibility baseline for release builds
- Automatic UI actions require the game window title `Teamfight Manager2`

## Building from source

The Mod SDK is not redistributed here. Install the matching SDK with the game, then run:

```powershell
.\build_local.ps1 -SdkDir "C:\path\to\Teamfight Manager2\mod-sdk"
```

The script reads the SDK's pinned Rust toolchain and produces `intro_skip.dll` in the repository root.

To validate and create a player-ready release archive:

```powershell
.\scripts\validate_repo.ps1
.\scripts\package_release.ps1 -SdkDir "C:\path\to\Teamfight Manager2\mod-sdk"
```

The archive is written to `builds\intro-skip-v0.4.7.zip`.

## Project layout

- `src/lib.rs` — startup automation, settings, backup, and recovery logic
- `ui/layout/title.ui` — title/Mods-screen UI override
- `mod.mod_info` — mod metadata and supported game range
- `mod.override_info` — asset remapping
- `settings.json` — source reference for the safe defaults; player settings are stored in AppData
- `thumbnail.png` — 256×256 lossless 2× nearest-neighbor pixel-art thumbnail
- `assets/thumbnail-master.png` — original 128×128 pixel-art thumbnail
- `assets/logo-1024.png` — official pixel-art logo master
- `assets/logo-{512,256,128,64,32}.png` — ready-to-use logo exports
- `assets/banner.png` — 1280×640 GitHub social preview
- `build_local.ps1` — SDK-aware native build
- `scripts/` — repository validation and release packaging

## Support

For a reproducible problem, open a bug report and attach:

- game version;
- Intro Skip version;
- enabled mod list;
- relevant lines from `backup_status.log`;
- the exact point where startup stopped.

Do **not** upload personal save files publicly.

## License and attribution

New versions of the original source code, scripts, and documentation are
released under the [Mozilla Public License 2.0](LICENSE). Distributed changes
to covered files must remain available under MPL-2.0. Earlier tagged releases
remain under the license shipped with those releases.

MPL-2.0 does not grant trademark rights in the Intro Skip name. The original
logo, banner, and thumbnail artwork are All Rights Reserved and are not covered
by MPL-2.0. The adapted Teamfight Manager 2 UI layout remains subject to Team
Samoyed's rights; see [NOTICE](NOTICE.md).

This is an independent community mod and is not affiliated with or endorsed by Team Samoyed.
