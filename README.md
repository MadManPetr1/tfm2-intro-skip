![Intro Skip banner](assets/banner.png)

<div align="center">

# Intro Skip

Safe, configurable startup automation for **Teamfight Manager 2**.

[Features](#features) · [Installation](#installation) · [Configuration](#configuration) · [Safety](#save-safety) · [Building](#building-from-source)

</div>

> [!IMPORTANT]
> Version **0.4.2** is built for Teamfight Manager 2 **0.5.0 only**. Compatibility with newer game versions has not yet been verified.

## Features

- Skips the startup disclaimer.
- Optionally selects **Continue** automatically.
- Optionally accepts the mod-state mismatch prompt.
- Creates and byte-verifies a save backup before automatic **Load Anyway**.
- Stops the automatic mismatch flow if backup creation or verification fails.
- Retains managed backups for seven days.
- Keeps recovery manual: a backup is imported into the regular **Load** menu only when you request it.
- Provides all settings directly inside the game's Mods screen.

| Setting | Included default |
| --- | --- |
| Skip disclaimer | On |
| Automatically select Continue | Off |
| Automatically Load Anyway on mod mismatch | Off |

## Installation

### Steam Workshop

Subscribe on the Teamfight Manager 2 Workshop, enable **Intro Skip** in the in-game Mods menu, then restart the game when prompted.

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
- **Import latest automatic backup into Load menu** creates a separate recovery save that you can select manually.

Changes are written outside the Workshop-managed mod folder so updates cannot replace them:

```text
%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip\settings.json
```

Existing `mods\intro_skip\settings.json` values are migrated automatically on first launch.

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

Managed backups older than seven days are removed. Diagnostic messages are appended to:

```text
%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip\backup_status.log
```

> [!WARNING]
> No mod can guarantee save compatibility when a career's enabled mods differ. The backup guard reduces recovery risk; it does not make incompatible mod combinations safe.

## Requirements and limitations

- Teamfight Manager 2 `0.5.0`
- Windows
- The matching `0.5.0` Mod SDK for source builds
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

The archive is written to `builds\intro-skip-v0.4.2.zip`.

## Project layout

- `src/lib.rs` — startup automation, settings, backup, and recovery logic
- `ui/layout/title.ui` — title/Mods-screen UI override
- `mod.mod_info` — mod metadata and supported game range
- `mod.override_info` — asset remapping
- `settings.json` — source reference for the safe defaults; player settings are stored in AppData
- `thumbnail.png` — 512×512 in-game/Workshop thumbnail
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

Original Rust code, scripts, documentation, and original project artwork are released under the [MIT License](LICENSE). The adapted Teamfight Manager 2 UI layout remains subject to Team Samoyed's rights; see [NOTICE](NOTICE.md).

This is an independent community mod and is not affiliated with or endorsed by Team Samoyed.
