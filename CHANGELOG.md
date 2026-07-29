# Changelog

All notable public changes to Intro Skip will be documented here.

The project uses [Semantic Versioning](https://semver.org/) for mod releases.

## [Unreleased]

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

[Unreleased]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.6...HEAD
[0.4.6]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/MadManPetr1/tfm2-intro-skip/tree/v0.4.1
