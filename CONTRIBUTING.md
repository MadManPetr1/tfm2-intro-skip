# Contributing

Bug reports, compatibility findings, documentation improvements, and focused pull requests are welcome.

## Before opening an issue

1. Confirm the game and mod versions.
2. Reproduce with the smallest practical enabled-mod list.
3. Check `mods\intro_skip\backup_status.log`.
4. Remove private information and never attach career saves publicly.

## Pull requests

- Keep changes focused.
- Run `cargo fmt --check`.
- Run `.\scripts\validate_repo.ps1` on Windows.
- Explain any behavior or save-handling change.
- Update `CHANGELOG.md` under **Unreleased**.

The native mod depends on the matching Teamfight Manager 2 Mod SDK, which is not redistributed in this repository.
