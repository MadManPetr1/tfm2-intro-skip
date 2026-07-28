# Contributing

Bug reports, compatibility findings, documentation improvements, and focused pull requests are welcome.

## Contribution terms

By submitting a contribution, you agree to license it under the
[Mozilla Public License 2.0](LICENSE). Submit only work you created or have the
right to contribute.

MPL-2.0 does not grant trademark rights in the Intro Skip name. The original
logo, banner, and thumbnail artwork are All Rights Reserved. Forks should use
their own name and artwork.

## Before opening an issue

1. Confirm the game and mod versions.
2. Reproduce with the smallest practical enabled-mod list.
3. Check `%APPDATA%\TeamSamoyed\TeamfightManager2\data\intro_skip\backup_status.log`.
4. Remove private information and never attach career saves publicly.

## Pull requests

- Keep changes focused.
- Run `cargo fmt --check`.
- Run `.\scripts\validate_repo.ps1` on Windows.
- Explain any behavior or save-handling change.
- Update `CHANGELOG.md` under **Unreleased**.
- Keep the default configuration safe and opt-in for save-affecting automation.

The native mod depends on the matching Teamfight Manager 2 Mod SDK, which is not redistributed in this repository.
