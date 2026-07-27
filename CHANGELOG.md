# Changelog

All notable public changes to Intro Skip will be documented here.

The project uses [Semantic Versioning](https://semver.org/) for mod releases.

## [Unreleased]

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

[Unreleased]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/MadManPetr1/tfm2-intro-skip/tree/v0.4.1
