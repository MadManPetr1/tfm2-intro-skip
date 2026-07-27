# Release and publishing guide

## 1. Pre-release verification

- Test the exact target game version.
- Test each setting independently.
- Confirm automatic mismatch loading stops when backup creation is forced to fail.
- Confirm a valid backup is byte-identical to the source.
- Confirm expired managed backups are removed while unrelated files remain.
- Confirm manual import creates a separate Load-menu entry.
- Confirm the mod works at multiple window sizes and display scales.
- Run `.\scripts\validate_repo.ps1`.

Do not broaden the version range in `mod.mod_info` until that game version has been tested.

## 2. Create the player archive

From PowerShell:

```powershell
.\scripts\package_release.ps1 -SdkDir "C:\path\to\Teamfight Manager2\mod-sdk"
```

Inspect `builds\intro-skip-vX.Y.Z.zip`. Its top-level folder must be `intro_skip`, and that folder must contain `intro_skip.dll`.

## 3. Create the GitHub repository

Recommended settings:

- Owner: `MadManPetr1`
- Repository: `TFM2---Intro-Skip`
- Visibility: Public
- Initialize with README: No
- Add `.gitignore`: No
- Add license: No

The repository package already includes these files.

```powershell
git init
git add .
git commit -m "Release Intro Skip v0.4.2"
git branch -M main
git remote add origin https://github.com/MadManPetr1/tfm2-intro-skip.git
git push -u origin main
```

Then configure:

- **About:** paste the description and topics from `docs/PRESENTATION.md`.
- **Social preview:** upload `assets/banner.png`.
- **Features:** keep Issues enabled; Discussions is optional.
- **Security:** enable private vulnerability reporting.

## 4. Publish GitHub v0.4.2

1. Open **Releases → Draft a new release**.
2. Create tag `v0.4.2` targeting `main`.
3. Paste the release title and notes from `docs/PRESENTATION.md`.
4. Attach `builds\intro-skip-v0.4.2.zip`.
5. Mark it as the latest release.
6. Publish.

## 5. Publish to Steam Workshop

1. Copy the packaged `intro_skip` folder to `Teamfight Manager2\mods\intro_skip`.
2. Start Steam and open `TFM2ModUploader.exe` from the game directory.
3. Select that exact runtime folder.
4. Add the title and description from `docs/PRESENTATION.md`.
5. Add the real in-game screenshots from the screenshot plan.
6. Choose visibility and publish.
7. Add the Workshop URL to the GitHub repository's Website field.

For the first upload, use the change note:

```text
Initial public release: configurable startup skipping, guarded automatic Continue/Load Anyway, verified save backups, and manual recovery import.
```

## 6. Final public check

Install once from GitHub Releases and once through Workshop on a clean mod folder. Confirm both installations show version `0.4.2`, include the DLL, load the settings panel, and retain the safe defaults.
