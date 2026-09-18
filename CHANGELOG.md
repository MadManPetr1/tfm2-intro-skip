# Changelog

All notable public changes to Intro Skip will be documented here.

The project uses [Semantic Versioning](https://semver.org/) for mod releases.

## [Unreleased]

## [0.6.1] - 2026-09-19

### Fixed

- Removed the early title overlay that appeared interactive while the game was
  still waiting for its scene-owned disclaimer timer.
- Removed the unavailable disclaimer toggle from Better Mod Menu on TFM2 0.6.0.
- Kept automatic Continue, verified Load Anyway, retention, and recovery active.

### Compatibility

- The TFM2 0.6.0 Stable API does not expose disclaimer progress, so this
  release waits for the native disclaimer instead of presenting frozen controls.

## [0.6.0] - 2026-09-19

### Changed

- Migrated Intro Skip from the retired classic API to the supported Stable Mod API.
- Reimplemented the disclaimer skip by mounting the game's native title layout
  without removing scene-owned UI state.
- Migrated Continue and verified Load Anyway automation to Stable UI queries.
- Preserved existing settings, managed backups, and Better Mod Menu integration.

### Compatibility

- Tested on Teamfight Manager 2 `0.6.0` with the Stable SDK.

## [0.5.5] - 2026-09-02

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.8` Mod SDK.
- Updated the declared base range to `>=0.5.8, <0.5.9`.

### Compatibility

- Tested on Teamfight Manager 2 `0.5.8`; existing settings and managed backups
  remain compatible.

## [0.5.4] - 2026-08-27

### Fixed

- Updated the Better Mod Menu runtime-surface contract so Intro Skip settings
  remain synchronized in the current menu.
- Stopped Intro Skip from rewriting shared native details text while Better Mod
  Menu owns the selected-mod panel.
- Hid only Intro Skip-owned native settings nodes while Better Mod Menu is open,
  preventing panel ownership conflicts without affecting standalone settings.

### Compatibility

- Built and manually tested with Better Mod Menu `0.7.5` on Teamfight Manager 2
  `0.5.7`; startup settings and existing managed backups remain compatible.

## [0.5.3] - 2026-08-26

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.7` Mod SDK.
- Restricted the declared base range to `>=0.5.7, <0.5.8` because the native
  SDK libraries changed and older game versions have not been runtime-tested
  with this build.

### Compatibility

- Tested on Teamfight Manager 2 `0.5.7` while preserving existing settings and
  managed backups.

## [0.5.2] - 2026-08-20

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.6` Mod SDK.
- Restricted the declared base range to `>=0.5.6, <0.5.7` because the native
  SDK libraries changed and older game versions have not been runtime-tested
  with this build.

### Compatibility

- Tested on Teamfight Manager 2 `0.5.6` while preserving existing settings and
  managed backups.

## [0.5.1] - 2026-08-12

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.5` Mod SDK.
- Restricted the declared base range to `>=0.5.5, <0.5.6` because the native
  SDK libraries changed and older game versions have not been runtime-tested
  with this build.

### Compatibility

- Tested on Teamfight Manager 2 `0.5.5` while preserving existing settings and
  managed backups.

## [0.5.0] - 2026-08-05

### Changed

- Standardized the Cargo crate, mod ID, installed folder, and DLL as
  `tfm2_intro_skip`.
- Preserved automatic migration from the previous `intro_skip` settings paths.

### Compatibility

- Runtime identity cleanup; existing managed backups remain in the shared
  `intro_skip_backups` directory.

## [0.4.9] - 2026-08-05

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.4` Mod SDK.
- Extended the supported base range to `>=0.5.2, <0.5.5` while preserving
  existing settings and backup data.

### Compatibility

- Runtime-tested on Teamfight Manager 2 `0.5.4`.

## [0.4.8] - 2026-08-03

### Added

- Unified Better Mod Menu author profile and profile icon.
- Refreshed 256 px pixel-art thumbnail and release banner.

### Changed

- Unified the pixel-art source assets and removed redundant logo-size and
  banner copies.
- Simplified player documentation and removed repository-layout details.

### Compatibility

- Runtime behavior, settings, backups, and the tested TFM2 `0.5.2`-`0.5.3`
  range are unchanged.

## [0.4.7] - 2026-07-29

### Changed

- Rebuilt the native DLL against the Teamfight Manager 2 `0.5.2` Mod SDK as
  the compatibility baseline.
- Expanded the supported game range to `>=0.5.2, <0.5.4`.
- Added optional Better Mod Menu controls and recent-backup actions while
  preserving the standalone native settings panel.
- Added the release banner to the player package for rich Better Mod Menu
  previews.
- Limited Better Mod Menu settings/action polling to the visible mod-manager
  surface instead of unrelated scenes.

### Compatibility

- Verified that the exact `0.5.2`-baseline DLL loads, registers, and reaches
  the rendered title screen on Teamfight Manager 2 `0.5.3`.

## [0.4.6] - 2026-07-29

### Changed

- Rebuilt the native mod against the Teamfight Manager 2 `0.5.3` Mod SDK.
- Updated the supported game range to `>=0.5.3, <0.5.4`.
- Updated the local build wrapper to link the SDK's LLVM bitcode with its
  pinned Rust LLVM linker.

### Performance

- Avoid repeated full settings-panel UI tree updates while another mod is
  selected.
- Reuse the native game-window title and keep save verification buffers off
  the game thread's stack.

### Fixed

- Guard automatic clicks while the game window has no usable client area,
  preventing an invalid coordinate clamp while minimized.
- Always attempt to release the synthetic mouse button if a preceding window
  message fails.

## [0.4.5] - 2026-07-28

### Changed

- Standardized the public documentation and contribution guidance across the MadManPetr1 TFM2 mod collection.
- Relicensed new project versions under MPL-2.0 so distributed changes to covered files remain shareable.
- Clarified that the original project artwork remains outside the source-code
  license.
- Updated the pixel-art thumbnail with the creator-provided revision.

## [0.4.4] - 2026-07-28

### Changed

- Rebuilt the native mod against the Teamfight Manager 2 `0.5.2` Mod SDK.
- Updated the supported game range to `>=0.5.2, <0.5.3`.

### Fixed

- Size the Intro Skip settings card to contain all recovery controls and restore the base description card to its original height.

## [0.4.3] - 2026-07-27

### Added

- Configurable backup retention with 3, 7, 14, and 30 day choices.
- A selectable newest-first list of the five most recent verified backups.
- Visible status feedback for settings, retention cleanup, imports, and failures.

### Changed

- Moved Intro Skip's version and base dependency range into the top identity card and removed the redundant bottom cards from its settings view.

## [0.4.2] - 2026-07-27

### Changed

- Rebuilt the Mods-screen settings area as a structured card with clear full-width controls and readable state hierarchy.

### Fixed

- Handle settings clicks with guarded control hit-testing because the 0.5.0 SDK does not dispatch events for controls injected into the overridden title layout.
- Hide the underlying description scroll view while showing Intro Skip settings, preventing it from intercepting configuration clicks.
- Treat each visible ON/OFF control as an explicit setting value so repeated SDK callbacks cannot cancel a change.
- Keep automatic Continue off when the settings file is missing or cannot be read, matching the published safe default.
- Store mutable settings and status logs in AppData so Workshop updates cannot replace user configuration.

## [0.4.1] - 2026-07-27

### Added

- Configurable disclaimer skipping.
- Optional automatic Continue selection.
- Optional automatic Load Anyway handling for mod-state mismatches.
- Verified pre-load save backups with seven-day retention.
- Manual recovery import through the in-game Mods screen.
- In-game settings panel and persistent JSON configuration.
- SDK-aware local build and release packaging scripts.
- Official pixel-art logo, small-size exports, and GitHub social-preview artwork.

### Safety

- Automatic Load Anyway is blocked unless the latest save has been copied and byte-verified.
- Recovery import always creates a separate save and remains a manual action.

[Unreleased]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.5.4...HEAD
[0.5.4]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.9...v0.5.0
[0.4.9]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.8...v0.4.9
[0.4.8]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.7...v0.4.8
[0.4.7]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/MadManPetr1/tfm2-intro-skip/tree/v0.4.1
