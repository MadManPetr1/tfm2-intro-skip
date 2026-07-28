# Release guide

## 1. Verify

- Test the exact target game version.
- Test each setting independently.
- Confirm automatic mismatch loading stops when backup creation is forced to fail.
- Confirm a valid backup is byte-identical to the source.
- Confirm expired managed backups are removed while unrelated files remain.
- Confirm manual import creates a separate Load-menu entry.
- Confirm the mod works at multiple window sizes and display scales.
- Run `.\scripts\validate_repo.ps1`.

Do not broaden the version range in `mod.mod_info` until that game version has
been tested.

## 2. Package

```powershell
.\scripts\package_release.ps1 -SdkDir "C:\path\to\Teamfight Manager2\mod-sdk"
```

Inspect `builds\intro-skip-vX.Y.Z.zip`. Its top-level folder must be
`intro_skip`, and that folder must contain `intro_skip.dll`.

## 3. Check the repository

Confirm the public repository is `MadManPetr1/tfm2-intro-skip`, Issues are
enabled, private vulnerability reporting is available, and the description and
topics match `docs/PRESENTATION.md`.

## 4. Publish

- Tag the exact source used to build the archive.
- Paste the release title and notes from `docs/PRESENTATION.md`.
- Attach the matching `builds\intro-skip-vX.Y.Z.zip` archive.
- Install the archive once into a clean mod folder before publishing.
- Keep the Workshop and GitHub version numbers aligned.

For the Workshop listing, use the title, description, and screenshot plan from
`docs/PRESENTATION.md`. The first-upload change note can be:

```text
Initial public release: configurable startup skipping, guarded automatic Continue/Load Anyway, verified save backups, and manual recovery import.
```

## 5. Final public check

Install once from GitHub Releases and once through Workshop on a clean mod
folder. Confirm both installations show the released version, include the DLL,
load the settings panel, and retain the safe defaults.
