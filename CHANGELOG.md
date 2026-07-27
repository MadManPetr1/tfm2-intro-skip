# Changelog

All notable public changes to Intro Skip will be documented here.

The project uses [Semantic Versioning](https://semver.org/) for mod releases.

## [Unreleased]

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

[Unreleased]: https://github.com/MadManPetr1/tfm2-intro-skip/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/MadManPetr1/tfm2-intro-skip/releases/tag/v0.4.1
