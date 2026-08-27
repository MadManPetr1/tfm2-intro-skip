use mod_api::*;
use std::ffi::c_void;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MOD_ID: &str = "tfm2_intro_skip";
const LEGACY_MOD_ID: &str = "intro_skip";
const MOD_NAME: &str = "Intro Skip";
const SETTINGS_PANEL_ID: &str = "intro_skip_settings";
const BMM_SURFACE_ID: &str = "bmm_surface";
const GAME_WINDOW_TITLE: &[u16] = &[
    84, 101, 97, 109, 102, 105, 103, 104, 116, 32, 77, 97, 110, 97, 103, 101, 114, 50, 0,
];

const DISCLAIMER_ON_ID: &str = "intro_skip_disclaimer_on";
const DISCLAIMER_OFF_ID: &str = "intro_skip_disclaimer_off";
const CONTINUE_ON_ID: &str = "intro_skip_continue_on";
const CONTINUE_OFF_ID: &str = "intro_skip_continue_off";
const FORCE_ON_ID: &str = "intro_skip_force_on";
const FORCE_OFF_ID: &str = "intro_skip_force_off";
const RETENTION_3_ON_ID: &str = "intro_skip_retention_3_on";
const RETENTION_3_OFF_ID: &str = "intro_skip_retention_3_off";
const RETENTION_7_ON_ID: &str = "intro_skip_retention_7_on";
const RETENTION_7_OFF_ID: &str = "intro_skip_retention_7_off";
const RETENTION_14_ON_ID: &str = "intro_skip_retention_14_on";
const RETENTION_14_OFF_ID: &str = "intro_skip_retention_14_off";
const RETENTION_30_ON_ID: &str = "intro_skip_retention_30_on";
const RETENTION_30_OFF_ID: &str = "intro_skip_retention_30_off";
const BACKUP_EMPTY_ID: &str = "intro_skip_backup_empty";
const BACKUP_BUTTON_IDS: [&str; 5] = [
    "intro_skip_backup_1",
    "intro_skip_backup_2",
    "intro_skip_backup_3",
    "intro_skip_backup_4",
    "intro_skip_backup_5",
];
const STATUS_READY_ID: &str = "intro_skip_status_ready";
const STATUS_SAVED_ID: &str = "intro_skip_status_saved";
const STATUS_RETENTION_ID: &str = "intro_skip_status_retention";
const STATUS_IMPORT_SUCCESS_ID: &str = "intro_skip_status_import_success";
const STATUS_FAILURE_ID: &str = "intro_skip_status_failure";

const DEFAULT_RETENTION_DAYS: u8 = 7;
const STATUS_NOTICE_FRAMES: usize = 300;
const STATUS_READY: u8 = 0;
const STATUS_SAVED: u8 = 1;
const STATUS_RETENTION: u8 = 2;
const STATUS_IMPORT_SUCCESS: u8 = 3;
const STATUS_FAILURE: u8 = 4;

#[derive(Clone, Copy)]
struct Settings {
    skip_disclaimer: bool,
    auto_continue: bool,
    auto_load_anyway: bool,
    backup_retention_days: u8,
}

fn should_sync_native_settings_panel(
    better_mod_menu_active: bool,
    show_settings: bool,
    was_showing_settings: bool,
) -> bool {
    !better_mod_menu_active && (show_settings || was_showing_settings)
}

#[derive(Clone, Copy)]
enum SettingKey {
    SkipDisclaimer,
    AutoContinue,
    AutoLoadAnyway,
}

impl SettingKey {
    fn store(self, settings: &RuntimeSettings, value: bool) {
        match self {
            Self::SkipDisclaimer => settings.skip_disclaimer.store(value, Ordering::Release),
            Self::AutoContinue => settings.auto_continue.store(value, Ordering::Release),
            Self::AutoLoadAnyway => settings.auto_load_anyway.store(value, Ordering::Release),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            skip_disclaimer: true,
            auto_continue: false,
            auto_load_anyway: false,
            backup_retention_days: DEFAULT_RETENTION_DAYS,
        }
    }
}

struct RuntimeSettings {
    skip_disclaimer: AtomicBool,
    auto_continue: AtomicBool,
    auto_load_anyway: AtomicBool,
    backup_retention_days: AtomicU8,
}

impl RuntimeSettings {
    fn new(settings: Settings) -> Self {
        Self {
            skip_disclaimer: AtomicBool::new(settings.skip_disclaimer),
            auto_continue: AtomicBool::new(settings.auto_continue),
            auto_load_anyway: AtomicBool::new(settings.auto_load_anyway),
            backup_retention_days: AtomicU8::new(settings.backup_retention_days),
        }
    }

    fn snapshot(&self) -> Settings {
        Settings {
            skip_disclaimer: self.skip_disclaimer.load(Ordering::Acquire),
            auto_continue: self.auto_continue.load(Ordering::Acquire),
            auto_load_anyway: self.auto_load_anyway.load(Ordering::Acquire),
            backup_retention_days: self.backup_retention_days.load(Ordering::Acquire),
        }
    }

    fn apply(&self, settings: Settings) {
        self.skip_disclaimer
            .store(settings.skip_disclaimer, Ordering::Release);
        self.auto_continue
            .store(settings.auto_continue, Ordering::Release);
        self.auto_load_anyway
            .store(settings.auto_load_anyway, Ordering::Release);
        self.backup_retention_days
            .store(settings.backup_retention_days, Ordering::Release);
    }
}

struct StatusNotice {
    status: AtomicU8,
    remaining_frames: AtomicUsize,
}

impl StatusNotice {
    fn new() -> Self {
        Self {
            status: AtomicU8::new(STATUS_READY),
            remaining_frames: AtomicUsize::new(0),
        }
    }

    fn show(&self, status: u8) {
        self.status.store(status, Ordering::Release);
        self.remaining_frames
            .store(STATUS_NOTICE_FRAMES, Ordering::Release);
    }

    fn current_and_tick(&self) -> u8 {
        let remaining = self.remaining_frames.load(Ordering::Acquire);
        if remaining == 0 {
            self.status.store(STATUS_READY, Ordering::Release);
            return STATUS_READY;
        }
        self.remaining_frames.fetch_sub(1, Ordering::AcqRel);
        self.status.load(Ordering::Acquire)
    }
}

struct IntroSkipExtension {
    settings: Arc<RuntimeSettings>,
    status_notice: Arc<StatusNotice>,
    backup_count: AtomicU8,
    settings_panel_visible: AtomicBool,
    title_ready_frames: AtomicUsize,
    auto_continue_sent: AtomicBool,
    mismatch_ready_frames: AtomicUsize,
    mismatch_click_sent: AtomicBool,
    mismatch_backup_ready: AtomicBool,
    mismatch_backup_failed: AtomicBool,
    retention_checked: AtomicBool,
    settings_mouse_down: AtomicBool,
    settings_reload_tick: AtomicUsize,
}

impl ModExtension for IntroSkipExtension {
    fn on_init(&self, scene: &mut Scene, _ui: &mut GameUI, _assets: &mut Assets) {
        if self.settings.skip_disclaimer.load(Ordering::Acquire) {
            skip_disclaimer(scene);
        }
    }

    fn post_update(&self, scene: &mut Scene, ui: &mut GameUI, assets: &mut Assets, _dt: f32) {
        if !self.retention_checked.swap(true, Ordering::AcqRel) {
            let _ = cleanup_expired_backups(
                self.settings.backup_retention_days.load(Ordering::Acquire),
            );
        }

        if self.settings.skip_disclaimer.load(Ordering::Acquire) {
            skip_disclaimer(scene);
        }

        let Scene::Title { disclaimer: None } = scene else {
            return;
        };

        let better_mod_menu_active = node_is_visible(&ui.root, BMM_SURFACE_ID);
        if better_mod_menu_active {
            let reload_tick = self.settings_reload_tick.fetch_add(1, Ordering::Relaxed);
            if reload_tick.is_multiple_of(30) {
                self.settings.apply(load_settings());
                self.handle_better_mod_menu_action();
            }
        } else {
            self.settings_reload_tick.store(0, Ordering::Relaxed);
        }
        let settings_mouse_pressed = !better_mod_menu_active && self.left_mouse_pressed();
        let show_settings = !better_mod_menu_active && selected_mod_is_intro_skip(&ui.root, assets);
        let was_showing_settings = self
            .settings_panel_visible
            .swap(show_settings, Ordering::AcqRel);
        if show_settings && !was_showing_settings {
            self.refresh_backup_count();
        }
        let status = self.status_notice.current_and_tick();
        if better_mod_menu_active {
            hide_intro_skip_settings_panel(&mut ui.root);
        } else if should_sync_native_settings_panel(
            better_mod_menu_active,
            show_settings,
            was_showing_settings,
        ) {
            sync_settings_panel(
                &mut ui.root,
                self.settings.snapshot(),
                show_settings,
                status,
                self.backup_count.load(Ordering::Acquire),
            );
        }
        if show_settings && settings_mouse_pressed {
            self.handle_settings_mouse_click(ui);
        }
        self.try_auto_continue(ui);
        self.try_auto_load_anyway(ui);
    }
}

impl IntroSkipExtension {
    fn handle_better_mod_menu_action(&self) {
        let Some(path) = better_mod_menu_actions_path() else {
            return;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            return;
        };
        let action = read_json_string(&source, "action").unwrap_or_default();
        let status = match action.as_str() {
            "import_backup" | "import_latest_backup" => {
                let index = read_json_u8(&source, "index").unwrap_or(0) as usize;
                match import_backup(index) {
                    Ok(imported) => {
                        record_backup_status(&format!(
                            "Imported backup {} into the Load menu: {}",
                            index + 1,
                            imported.display()
                        ));
                        STATUS_IMPORT_SUCCESS
                    }
                    Err(error) => {
                        record_backup_status(&format!("Backup import failed: {error}"));
                        STATUS_FAILURE
                    }
                }
            }
            _ => STATUS_FAILURE,
        };
        let _ = fs::remove_file(path);
        self.status_notice.show(status);
        self.refresh_backup_count();
    }

    fn left_mouse_pressed(&self) -> bool {
        const VK_LBUTTON: i32 = 0x01;
        let is_down = unsafe { GetAsyncKeyState(VK_LBUTTON) } < 0;
        let was_down = self.settings_mouse_down.swap(is_down, Ordering::AcqRel);
        is_down && !was_down
    }

    fn handle_settings_mouse_click(&self, ui: &GameUI) {
        let Some((ui_x, ui_y)) = cursor_ui_position(ui) else {
            return;
        };

        for (id, key, value) in [
            (DISCLAIMER_ON_ID, SettingKey::SkipDisclaimer, false),
            (DISCLAIMER_OFF_ID, SettingKey::SkipDisclaimer, true),
            (CONTINUE_ON_ID, SettingKey::AutoContinue, false),
            (CONTINUE_OFF_ID, SettingKey::AutoContinue, true),
            (FORCE_ON_ID, SettingKey::AutoLoadAnyway, false),
            (FORCE_OFF_ID, SettingKey::AutoLoadAnyway, true),
        ] {
            if node_contains_point(&ui.root, id, ui_x, ui_y) {
                key.store(&self.settings, value);
                let status = if save_runtime_settings(&self.settings).is_ok() {
                    STATUS_SAVED
                } else {
                    STATUS_FAILURE
                };
                self.status_notice.show(status);
                return;
            }
        }

        for (selected_id, unselected_id, days) in [
            (RETENTION_3_ON_ID, RETENTION_3_OFF_ID, 3),
            (RETENTION_7_ON_ID, RETENTION_7_OFF_ID, 7),
            (RETENTION_14_ON_ID, RETENTION_14_OFF_ID, 14),
            (RETENTION_30_ON_ID, RETENTION_30_OFF_ID, 30),
        ] {
            if node_contains_point(&ui.root, selected_id, ui_x, ui_y)
                || node_contains_point(&ui.root, unselected_id, ui_x, ui_y)
            {
                self.settings
                    .backup_retention_days
                    .store(days, Ordering::Release);
                let saved = save_runtime_settings(&self.settings).is_ok();
                let cleaned = cleanup_expired_backups(days).is_ok();
                self.refresh_backup_count();
                self.status_notice.show(if saved && cleaned {
                    STATUS_RETENTION
                } else {
                    STATUS_FAILURE
                });
                return;
            }
        }

        for (index, id) in BACKUP_BUTTON_IDS.iter().enumerate() {
            if !node_contains_point(&ui.root, id, ui_x, ui_y) {
                continue;
            }
            match import_backup(index) {
                Ok(path) => {
                    record_backup_status(&format!(
                        "Imported backup {} into the Load menu: {}",
                        index + 1,
                        path.display()
                    ));
                    self.status_notice.show(STATUS_IMPORT_SUCCESS);
                }
                Err(error) => {
                    record_backup_status(&format!("Backup import failed: {error}"));
                    self.status_notice.show(STATUS_FAILURE);
                }
            }
            return;
        }
    }

    fn refresh_backup_count(&self) {
        let count = backup_dir()
            .and_then(|directory| recent_backups(&directory, BACKUP_BUTTON_IDS.len()).ok())
            .map_or(0, |backups| backups.len() as u8);
        self.backup_count.store(count, Ordering::Release);
    }

    fn try_auto_continue(&self, ui: &GameUI) {
        if !self.settings.auto_continue.load(Ordering::Acquire)
            || self.auto_continue_sent.load(Ordering::Acquire)
            || !node_is_visible(&ui.root, "continue")
        {
            return;
        }

        if self.title_ready_frames.fetch_add(1, Ordering::Relaxed) < 2 {
            return;
        }

        if post_node_click(ui, "continue") {
            self.auto_continue_sent.store(true, Ordering::Release);
        }
    }

    fn try_auto_load_anyway(&self, ui: &GameUI) {
        let popup_visible = node_is_visible(&ui.root, "mod_compat_popup");
        if !popup_visible {
            self.mismatch_ready_frames.store(0, Ordering::Release);
            self.mismatch_click_sent.store(false, Ordering::Release);
            self.mismatch_backup_ready.store(false, Ordering::Release);
            self.mismatch_backup_failed.store(false, Ordering::Release);
            return;
        }

        if !self.settings.auto_load_anyway.load(Ordering::Acquire)
            || self.mismatch_click_sent.load(Ordering::Acquire)
        {
            return;
        }

        if self.mismatch_ready_frames.fetch_add(1, Ordering::Relaxed) < 2 {
            return;
        }

        if !self.mismatch_backup_ready.load(Ordering::Acquire)
            && !self.mismatch_backup_failed.load(Ordering::Acquire)
        {
            match backup_latest_save(self.settings.backup_retention_days.load(Ordering::Acquire)) {
                Ok(path) => {
                    record_backup_status(&format!("Backup ready: {}", path.display()));
                    self.mismatch_backup_ready.store(true, Ordering::Release);
                }
                Err(error) => {
                    record_backup_status(&format!(
                        "Automatic Load Anyway stopped because backup failed: {error}"
                    ));
                    self.mismatch_backup_failed.store(true, Ordering::Release);
                }
            }
        }

        if !self.mismatch_backup_ready.load(Ordering::Acquire) {
            return;
        }

        if post_node_click(ui, "force") {
            self.mismatch_click_sent.store(true, Ordering::Release);
        }
    }
}

fn skip_disclaimer(scene: &mut Scene) {
    if let Scene::Title {
        disclaimer: Some(progress),
    } = scene
    {
        *progress = 1.0;
    }
}

fn find_node<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    if node.id == id {
        return Some(node);
    }
    node.child.iter().find_map(|child| find_node(child, id))
}

fn find_visible_node<'a>(node: &'a Node, id: &str, ancestors_visible: bool) -> Option<&'a Node> {
    let visible = ancestors_visible && node.visible;
    if node.id == id {
        return visible.then_some(node);
    }
    if !visible {
        return None;
    }
    node.child
        .iter()
        .find_map(|child| find_visible_node(child, id, visible))
}

fn node_is_visible(node: &Node, id: &str) -> bool {
    find_visible_node(node, id, true).is_some()
}

fn set_node_visible(node: &mut Node, id: &str, visible: bool) -> bool {
    if node.id == id {
        node.visible = visible;
        return true;
    }
    node.child
        .iter_mut()
        .any(|child| set_node_visible(child, id, visible))
}

fn node_contains_point(root: &Node, id: &str, x: f32, y: f32) -> bool {
    let Some(node) = find_visible_node(root, id, true) else {
        return false;
    };
    node.rect.w > 0.0
        && node.rect.h > 0.0
        && x >= node.rect.x
        && x <= node.rect.x + node.rect.w
        && y >= node.rect.y
        && y <= node.rect.y + node.rect.h
}

fn selected_mod_is_intro_skip(root: &Node, assets: &Assets) -> bool {
    let Some(name_node) = find_node(root, "mod_name") else {
        return false;
    };
    if name_node.runner.type_name() != "engine_ui::runner::label::LabelRunner" {
        return false;
    }
    // The SDK's Any downcast does not preserve the runner's concrete identity
    // across the game/mod boundary. Guard the cast with the SDK type name.
    let label =
        unsafe { &*(name_node.runner.as_ref() as *const dyn NodeRunner as *const LabelRunner) };
    label
        .rendered_text(assets)
        .is_some_and(|name| name == MOD_NAME)
}

fn sync_settings_panel(
    root: &mut Node,
    settings: Settings,
    show_settings: bool,
    status: u8,
    backup_count: u8,
) {
    set_node_visible(root, "description_panel", !show_settings);
    set_node_visible(root, SETTINGS_PANEL_ID, show_settings);
    set_node_visible(root, "intro_skip_header_version", show_settings);
    set_node_visible(root, "intro_skip_header_dependencies", show_settings);
    set_node_visible(root, "deps_panel", !show_settings);
    set_node_visible(root, "version_panel", !show_settings);
    set_node_visible(root, DISCLAIMER_ON_ID, settings.skip_disclaimer);
    set_node_visible(root, DISCLAIMER_OFF_ID, !settings.skip_disclaimer);
    set_node_visible(root, CONTINUE_ON_ID, settings.auto_continue);
    set_node_visible(root, CONTINUE_OFF_ID, !settings.auto_continue);
    set_node_visible(root, FORCE_ON_ID, settings.auto_load_anyway);
    set_node_visible(root, FORCE_OFF_ID, !settings.auto_load_anyway);
    for (selected_id, unselected_id, days) in [
        (RETENTION_3_ON_ID, RETENTION_3_OFF_ID, 3),
        (RETENTION_7_ON_ID, RETENTION_7_OFF_ID, 7),
        (RETENTION_14_ON_ID, RETENTION_14_OFF_ID, 14),
        (RETENTION_30_ON_ID, RETENTION_30_OFF_ID, 30),
    ] {
        let selected = settings.backup_retention_days == days;
        set_node_visible(root, selected_id, selected);
        set_node_visible(root, unselected_id, !selected);
    }
    for (index, id) in BACKUP_BUTTON_IDS.iter().enumerate() {
        set_node_visible(root, id, index < backup_count as usize);
    }
    set_node_visible(root, BACKUP_EMPTY_ID, backup_count == 0);
    set_node_visible(root, STATUS_READY_ID, status == STATUS_READY);
    set_node_visible(root, STATUS_SAVED_ID, status == STATUS_SAVED);
    set_node_visible(root, STATUS_RETENTION_ID, status == STATUS_RETENTION);
    set_node_visible(
        root,
        STATUS_IMPORT_SUCCESS_ID,
        status == STATUS_IMPORT_SUCCESS,
    );
    set_node_visible(root, STATUS_FAILURE_ID, status == STATUS_FAILURE);
}

fn hide_intro_skip_settings_panel(root: &mut Node) {
    set_node_visible(root, SETTINGS_PANEL_ID, false);
    set_node_visible(root, "intro_skip_header_version", false);
    set_node_visible(root, "intro_skip_header_dependencies", false);
}

fn game_data_dir() -> Option<PathBuf> {
    Some(
        PathBuf::from(std::env::var_os("APPDATA")?)
            .join("TeamSamoyed")
            .join("TeamfightManager2")
            .join("data"),
    )
}

fn backup_dir() -> Option<PathBuf> {
    Some(game_data_dir()?.join("intro_skip_backups"))
}

fn latest_save_in(directory: &std::path::Path) -> Result<PathBuf, String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if path.is_file() && name.starts_with("save_") && path.extension()?.to_str()? == "data"
            {
                let modified = entry.metadata().ok()?.modified().unwrap_or(UNIX_EPOCH);
                Some((modified, path))
            } else {
                None
            }
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
        .ok_or_else(|| format!("no save_*.data file found in {}", directory.display()))
}

fn recent_backups(directory: &std::path::Path, limit: usize) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    let mut backups: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !is_managed_backup(&path) {
                return None;
            }
            let modified = entry.metadata().ok()?.modified().unwrap_or(UNIX_EPOCH);
            Some((modified, path))
        })
        .collect();
    backups.sort_unstable_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    Ok(backups
        .into_iter()
        .take(limit)
        .map(|(_, path)| path)
        .collect())
}

fn latest_matching_backup(directory: &std::path::Path, source_stem: &str) -> Option<PathBuf> {
    let prefix = format!("{source_stem}__before_mod_load_");
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if path.is_file() && name.starts_with(&prefix) && path.extension()?.to_str()? == "data"
            {
                let modified = entry.metadata().ok()?.modified().unwrap_or(UNIX_EPOCH);
                Some((modified, path))
            } else {
                None
            }
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn is_managed_backup(path: &std::path::Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    path.is_file()
        && name.starts_with("save_")
        && name.contains("__before_mod_load_")
        && path.extension().and_then(|extension| extension.to_str()) == Some("data")
}

fn prune_expired_backups(directory: &std::path::Path, retention_days: u8) -> Result<usize, String> {
    let now = SystemTime::now();
    let retention = Duration::from_secs(u64::from(retention_days) * 24 * 60 * 60);
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    let mut removed = 0;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !is_managed_backup(&path) {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        let created = metadata.created().or_else(|_| metadata.modified());
        let Ok(created) = created else {
            continue;
        };
        let Ok(age) = now.duration_since(created) else {
            continue;
        };
        if age <= retention {
            continue;
        }
        fs::remove_file(&path)
            .map_err(|error| format!("cannot remove expired backup {}: {error}", path.display()))?;
        removed += 1;
    }
    Ok(removed)
}

fn cleanup_expired_backups(retention_days: u8) -> Result<usize, String> {
    let Some(backups) = backup_dir() else {
        return Err("APPDATA is unavailable".to_string());
    };
    if !backups.exists() {
        return Ok(0);
    }
    let result = prune_expired_backups(&backups, retention_days);
    match &result {
        Ok(removed) if *removed > 0 => {
            record_backup_status(&format!(
                "Removed {removed} backup(s) older than {retention_days} days"
            ));
        }
        Err(error) => {
            record_backup_status(&format!("Backup retention cleanup warning: {error}"));
        }
        _ => {}
    }
    result
}

fn backup_latest_save(retention_days: u8) -> Result<PathBuf, String> {
    let data_dir = game_data_dir().ok_or("APPDATA is unavailable")?;
    let source = latest_save_in(&data_dir)?;
    let backups = backup_dir().ok_or("APPDATA is unavailable")?;
    fs::create_dir_all(&backups)
        .map_err(|error| format!("cannot create {}: {error}", backups.display()))?;
    let _ = cleanup_expired_backups(retention_days);

    let source_metadata = fs::metadata(&source)
        .map_err(|error| format!("cannot inspect {}: {error}", source.display()))?;
    let source_stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("save filename is not valid UTF-8")?;

    if let Some(existing) = latest_matching_backup(&backups, source_stem) {
        if let Ok(metadata) = fs::metadata(&existing) {
            if metadata.len() == source_metadata.len()
                && metadata.modified().unwrap_or(UNIX_EPOCH)
                    >= source_metadata.modified().unwrap_or(SystemTime::now())
                && files_are_equal(&source, &existing)?
            {
                return Ok(existing);
            }
        }
    }

    let target = backups.join(format!(
        "{source_stem}__before_mod_load_{}.data",
        local_timestamp()
    ));
    copy_verified_atomic(&source, &target, "backup")?;
    Ok(target)
}

fn import_backup(index: usize) -> Result<PathBuf, String> {
    let backups = backup_dir().ok_or("APPDATA is unavailable")?;
    let source = recent_backups(&backups, BACKUP_BUTTON_IDS.len())?
        .into_iter()
        .nth(index)
        .ok_or_else(|| format!("backup {} is no longer available", index + 1))?;
    let data_dir = game_data_dir().ok_or("APPDATA is unavailable")?;
    let target = data_dir.join(format!("save_{}.data", local_timestamp()));
    if target.exists() {
        return Err(format!(
            "{} already exists; wait one second and try again",
            target.display()
        ));
    }
    copy_verified_atomic(&source, &target, "import")?;
    Ok(target)
}

fn copy_verified_atomic(
    source: &std::path::Path,
    target: &std::path::Path,
    operation: &str,
) -> Result<(), String> {
    if target.exists() {
        return Err(format!("{} already exists", target.display()));
    }
    let temporary = target.with_extension("data.intro_skip_tmp");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|error| {
            format!(
                "cannot clear stale temporary file {}: {error}",
                temporary.display()
            )
        })?;
    }

    let result = (|| {
        let expected = fs::metadata(source)
            .map_err(|error| format!("cannot inspect {}: {error}", source.display()))?
            .len();
        let copied = fs::copy(source, &temporary).map_err(|error| {
            format!(
                "cannot copy {} to {}: {error}",
                source.display(),
                temporary.display()
            )
        })?;
        if copied != expected {
            return Err(format!(
                "{operation} size mismatch: expected {expected} bytes, copied {copied}"
            ));
        }
        if !files_are_equal(source, &temporary)? {
            return Err(format!(
                "{operation} verification failed: copied bytes differ"
            ));
        }
        fs::rename(&temporary, target).map_err(|error| {
            format!(
                "cannot finalize {} as {}: {error}",
                temporary.display(),
                target.display()
            )
        })
    })();

    if result.is_err() && temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn files_are_equal(first: &std::path::Path, second: &std::path::Path) -> Result<bool, String> {
    let mut first_file = fs::File::open(first)
        .map_err(|error| format!("cannot open {}: {error}", first.display()))?;
    let mut second_file = fs::File::open(second)
        .map_err(|error| format!("cannot open {}: {error}", second.display()))?;
    let mut first_buffer = vec![0_u8; 64 * 1024];
    let mut second_buffer = vec![0_u8; 64 * 1024];

    loop {
        let first_read = first_file
            .read(&mut first_buffer)
            .map_err(|error| format!("cannot verify {}: {error}", first.display()))?;
        let second_read = second_file
            .read(&mut second_buffer)
            .map_err(|error| format!("cannot verify {}: {error}", second.display()))?;
        if first_read != second_read || first_buffer[..first_read] != second_buffer[..second_read] {
            return Ok(false);
        }
        if first_read == 0 {
            return Ok(true);
        }
    }
}

fn record_backup_status(message: &str) {
    append_status_log("backup_status.log", message);
}

fn record_runtime_status(message: &str) {
    append_status_log("runtime_status.log", message);
}

fn append_status_log(file_name: &str, message: &str) {
    let Some(directory) = state_dir() else {
        return;
    };
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let path = directory.join(file_name);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}: {message}", local_timestamp());
    }
}

fn state_dir() -> Option<PathBuf> {
    Some(game_data_dir()?.join(MOD_ID))
}

fn settings_path() -> Option<PathBuf> {
    Some(state_dir()?.join("settings.json"))
}

fn better_mod_menu_actions_path() -> Option<PathBuf> {
    Some(state_dir()?.join("better_mod_menu.actions.json"))
}

fn legacy_settings_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(data_dir) = game_data_dir() {
        paths.push(data_dir.join(LEGACY_MOD_ID).join("settings.json"));
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(game_dir) = executable.parent() {
            paths.push(game_dir.join("mods").join(MOD_ID).join("settings.json"));
            paths.push(
                game_dir
                    .join("mods")
                    .join(LEGACY_MOD_ID)
                    .join("settings.json"),
            );
        }
    }
    paths
}

fn load_settings() -> Settings {
    let mut settings = Settings::default();
    let Some(path) = settings_path() else {
        return settings;
    };

    let (source, mut should_persist) = match fs::read_to_string(&path) {
        Ok(source) => (source, false),
        Err(_) => match legacy_settings_paths()
            .into_iter()
            .find_map(|path| fs::read_to_string(path).ok())
        {
            Some(source) => (source, true),
            None => {
                let _ = write_settings(settings);
                return settings;
            }
        },
    };

    settings.skip_disclaimer =
        read_json_bool(&source, "skip_disclaimer").unwrap_or(settings.skip_disclaimer);
    settings.auto_continue =
        read_json_bool(&source, "auto_continue").unwrap_or(settings.auto_continue);
    settings.auto_load_anyway =
        read_json_bool(&source, "auto_load_anyway").unwrap_or(settings.auto_load_anyway);
    match read_json_u8(&source, "backup_retention_days")
        .filter(|days| matches!(days, 3 | 7 | 14 | 30))
    {
        Some(days) => settings.backup_retention_days = days,
        None => should_persist = true,
    }

    if should_persist {
        if let Err(error) = write_settings(settings) {
            eprintln!("intro_skip: failed to migrate settings: {error}");
        }
    }
    settings
}

fn write_settings(settings: Settings) -> Result<(), String> {
    let path = settings_path().ok_or("settings directory is unavailable")?;
    let Some(directory) = path.parent() else {
        return Err("settings directory is unavailable".to_string());
    };
    fs::create_dir_all(directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let source = format!(
        "{{\n  \"skip_disclaimer\": {},\n  \"auto_continue\": {},\n  \"auto_load_anyway\": {},\n  \"backup_retention_days\": {}\n}}\n",
        settings.skip_disclaimer,
        settings.auto_continue,
        settings.auto_load_anyway,
        settings.backup_retention_days
    );
    fs::write(&path, source).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn save_runtime_settings(settings: &RuntimeSettings) -> Result<(), String> {
    write_settings(settings.snapshot()).map_err(|error| {
        record_runtime_status(&format!("settings_save_failed error={error}"));
        eprintln!("intro_skip: failed to save settings: {error}");
        error
    })
}

fn read_json_bool(source: &str, key: &str) -> Option<bool> {
    let key = format!("\"{key}\"");
    let value = source.split_once(&key)?.1.split_once(':')?.1.trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn read_json_u8(source: &str, key: &str) -> Option<u8> {
    let key = format!("\"{key}\"");
    let value = source.split_once(&key)?.1.split_once(':')?.1.trim_start();
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn read_json_string(source: &str, key: &str) -> Option<String> {
    let key = format!("\"{key}\"");
    let value = source.split_once(&key)?.1.split_once(':')?.1.trim_start();
    let value = value.strip_prefix('"')?;
    let mut escaped = false;
    let mut output = String::new();
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '"' => return Some(output),
            _ => output.push(character),
        }
    }
    None
}

#[repr(C)]
struct ClientRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct SystemTimeWindows {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class_name: *const u16, window_name: *const u16) -> *mut c_void;
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
    fn GetClientRect(window: *mut c_void, rect: *mut ClientRect) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn PostMessageW(window: *mut c_void, message: u32, wparam: usize, lparam: isize) -> i32;
    fn ScreenToClient(window: *mut c_void, point: *mut Point) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(system_time: *mut SystemTimeWindows);
}

fn local_timestamp() -> String {
    let mut time = SystemTimeWindows {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut time) };
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        time.year, time.month, time.day, time.hour, time.minute, time.second
    )
}

fn post_node_click(ui: &GameUI, id: &str) -> bool {
    let Some(node) = find_node(&ui.root, id) else {
        return false;
    };
    if node.rect.w <= 0.0 || node.rect.h <= 0.0 || ui.rect.w <= 0.0 || ui.rect.h <= 0.0 {
        return false;
    }
    let ui_x = node.rect.x + node.rect.w * 0.5;
    let ui_y = node.rect.y + node.rect.h * 0.5;
    post_ui_click(ui, ui_x, ui_y)
}

fn game_window() -> *mut c_void {
    unsafe { FindWindowW(std::ptr::null(), GAME_WINDOW_TITLE.as_ptr()) }
}

fn cursor_ui_position(ui: &GameUI) -> Option<(f32, f32)> {
    let window = game_window();
    if window.is_null() {
        return None;
    }

    let mut point = Point { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut point) } == 0
        || unsafe { ScreenToClient(window, &mut point) } == 0
    {
        return None;
    }

    let mut rect = ClientRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetClientRect(window, &mut rect) } == 0 {
        return None;
    }

    let client_width = (rect.right - rect.left) as f32;
    let client_height = (rect.bottom - rect.top) as f32;
    if client_width <= 0.0 || client_height <= 0.0 || ui.rect.w <= 0.0 || ui.rect.h <= 0.0 {
        return None;
    }
    if point.x < rect.left || point.y < rect.top || point.x >= rect.right || point.y >= rect.bottom
    {
        return None;
    }

    Some((
        ui.rect.x + ((point.x - rect.left) as f32 / client_width) * ui.rect.w,
        ui.rect.y + ((point.y - rect.top) as f32 / client_height) * ui.rect.h,
    ))
}

fn post_ui_click(ui: &GameUI, ui_x: f32, ui_y: f32) -> bool {
    let window = game_window();
    if window.is_null() {
        return false;
    }

    let mut rect = ClientRect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetClientRect(window, &mut rect) } == 0 {
        return false;
    }

    let client_width = (rect.right - rect.left) as f32;
    let client_height = (rect.bottom - rect.top) as f32;
    if client_width <= 0.0 || client_height <= 0.0 || ui.rect.w <= 0.0 || ui.rect.h <= 0.0 {
        return false;
    }
    let x = (((ui_x - ui.rect.x) / ui.rect.w) * client_width)
        .clamp(0.0, client_width - 1.0)
        .round() as i32;
    let y = (((ui_y - ui.rect.y) / ui.rect.h) * client_height)
        .clamp(0.0, client_height - 1.0)
        .round() as i32;
    let position = ((y as isize) << 16) | ((x as isize) & 0xffff);

    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const MK_LBUTTON: usize = 0x0001;

    let moved = unsafe { PostMessageW(window, WM_MOUSEMOVE, 0, position) } != 0;
    let pressed = unsafe { PostMessageW(window, WM_LBUTTONDOWN, MK_LBUTTON, position) } != 0;
    let released = unsafe { PostMessageW(window, WM_LBUTTONUP, 0, position) } != 0;
    moved && pressed && released
}

fn init(_ctx: &GameCtx) -> ModRegistration {
    let settings = Arc::new(RuntimeSettings::new(load_settings()));
    let mut registration = ModRegistration::new(MOD_ID);
    registration.set_extension(IntroSkipExtension {
        settings,
        status_notice: Arc::new(StatusNotice::new()),
        backup_count: AtomicU8::new(0),
        settings_panel_visible: AtomicBool::new(false),
        title_ready_frames: AtomicUsize::new(0),
        auto_continue_sent: AtomicBool::new(false),
        mismatch_ready_frames: AtomicUsize::new(0),
        mismatch_click_sent: AtomicBool::new(false),
        mismatch_backup_ready: AtomicBool::new(false),
        mismatch_backup_failed: AtomicBool::new(false),
        retention_checked: AtomicBool::new(false),
        settings_mouse_down: AtomicBool::new(false),
        settings_reload_tick: AtomicUsize::new(0),
    });
    registration
}

declare_mod!(init);

#[cfg(test)]
mod tests {
    use super::{
        is_managed_backup, read_json_bool, read_json_string, read_json_u8,
        should_sync_native_settings_panel, BMM_SURFACE_ID,
    };
    use std::path::Path;

    #[test]
    fn reads_settings_and_better_mod_menu_actions() {
        let source = r#"{"skip_disclaimer":true,"index":2,"action":"import_backup"}"#;
        assert_eq!(read_json_bool(source, "skip_disclaimer"), Some(true));
        assert_eq!(read_json_u8(source, "index"), Some(2));
        assert_eq!(
            read_json_string(source, "action").as_deref(),
            Some("import_backup")
        );
    }

    #[test]
    fn uses_the_better_mod_menu_runtime_surface_contract() {
        assert_eq!(BMM_SURFACE_ID, "bmm_surface");
    }

    #[test]
    fn better_mod_menu_owns_native_text_visibility_while_open() {
        assert!(!should_sync_native_settings_panel(true, false, true));
        assert!(should_sync_native_settings_panel(false, true, false));
        assert!(should_sync_native_settings_panel(false, false, true));
        assert!(!should_sync_native_settings_panel(false, false, false));
    }

    #[test]
    fn recognizes_only_managed_backup_names() {
        let path = std::env::temp_dir().join(format!(
            "save_1__before_mod_load_20260730_0102_{}.data",
            std::process::id()
        ));
        std::fs::write(&path, b"test").expect("create managed-backup fixture");
        assert!(is_managed_backup(&path));
        std::fs::remove_file(&path).expect("remove managed-backup fixture");
        assert!(!is_managed_backup(Path::new("save_1.data")));
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
