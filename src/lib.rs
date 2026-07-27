use mod_api::*;
use std::ffi::c_void;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MOD_ID: &str = "intro_skip";
const MOD_NAME: &str = "Intro Skip";
const SETTINGS_PANEL_ID: &str = "intro_skip_settings";

const DISCLAIMER_ON_ID: &str = "intro_skip_disclaimer_on";
const DISCLAIMER_OFF_ID: &str = "intro_skip_disclaimer_off";
const CONTINUE_ON_ID: &str = "intro_skip_continue_on";
const CONTINUE_OFF_ID: &str = "intro_skip_continue_off";
const FORCE_ON_ID: &str = "intro_skip_force_on";
const FORCE_OFF_ID: &str = "intro_skip_force_off";
const IMPORT_BACKUP_ID: &str = "intro_skip_import_backup";
const IMPORT_DEFAULT_ID: &str = "intro_skip_import_default";
const IMPORT_SUCCESS_ID: &str = "intro_skip_import_success";
const IMPORT_FAILURE_ID: &str = "intro_skip_import_failure";

const BACKUP_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const IMPORT_NOTICE_FRAMES: usize = 300;
const IMPORT_NOTICE_NONE: u8 = 0;
const IMPORT_NOTICE_SUCCESS: u8 = 1;
const IMPORT_NOTICE_FAILURE: u8 = 2;

#[derive(Clone, Copy)]
struct Settings {
    skip_disclaimer: bool,
    auto_continue: bool,
    auto_load_anyway: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            skip_disclaimer: true,
            auto_continue: true,
            auto_load_anyway: false,
        }
    }
}

struct RuntimeSettings {
    skip_disclaimer: AtomicBool,
    auto_continue: AtomicBool,
    auto_load_anyway: AtomicBool,
}

impl RuntimeSettings {
    fn new(settings: Settings) -> Self {
        Self {
            skip_disclaimer: AtomicBool::new(settings.skip_disclaimer),
            auto_continue: AtomicBool::new(settings.auto_continue),
            auto_load_anyway: AtomicBool::new(settings.auto_load_anyway),
        }
    }

    fn snapshot(&self) -> Settings {
        Settings {
            skip_disclaimer: self.skip_disclaimer.load(Ordering::Acquire),
            auto_continue: self.auto_continue.load(Ordering::Acquire),
            auto_load_anyway: self.auto_load_anyway.load(Ordering::Acquire),
        }
    }
}

struct ImportNotice {
    status: AtomicU8,
    remaining_frames: AtomicUsize,
}

impl ImportNotice {
    fn new() -> Self {
        Self {
            status: AtomicU8::new(IMPORT_NOTICE_NONE),
            remaining_frames: AtomicUsize::new(0),
        }
    }

    fn show(&self, status: u8) {
        self.status.store(status, Ordering::Release);
        self.remaining_frames
            .store(IMPORT_NOTICE_FRAMES, Ordering::Release);
    }

    fn current_and_tick(&self) -> u8 {
        let remaining = self.remaining_frames.load(Ordering::Acquire);
        if remaining == 0 {
            self.status.store(IMPORT_NOTICE_NONE, Ordering::Release);
            return IMPORT_NOTICE_NONE;
        }
        self.remaining_frames.fetch_sub(1, Ordering::AcqRel);
        self.status.load(Ordering::Acquire)
    }
}

struct IntroSkipExtension {
    settings: Arc<RuntimeSettings>,
    import_notice: Arc<ImportNotice>,
    title_ready_frames: AtomicUsize,
    auto_continue_sent: AtomicBool,
    mismatch_ready_frames: AtomicUsize,
    mismatch_click_sent: AtomicBool,
    mismatch_backup_ready: AtomicBool,
    mismatch_backup_failed: AtomicBool,
    retention_checked: AtomicBool,
    handlers_registered: AtomicBool,
}

impl ModExtension for IntroSkipExtension {
    fn on_init(&self, scene: &mut Scene, _ui: &mut GameUI, _assets: &mut Assets) {
        if self.settings.skip_disclaimer.load(Ordering::Acquire) {
            skip_disclaimer(scene);
        }
    }

    fn post_update(&self, scene: &mut Scene, ui: &mut GameUI, assets: &mut Assets, _dt: f32) {
        if !self.retention_checked.swap(true, Ordering::AcqRel) {
            cleanup_expired_backups();
        }

        if self.settings.skip_disclaimer.load(Ordering::Acquire) {
            skip_disclaimer(scene);
        }

        let Scene::Title { disclaimer: None } = scene else {
            return;
        };

        if node_contains_id(&ui.root, SETTINGS_PANEL_ID)
            && !self.handlers_registered.swap(true, Ordering::AcqRel)
        {
            register_setting_handlers(
                ui,
                Arc::clone(&self.settings),
                Arc::clone(&self.import_notice),
            );
        }
        let show_settings = selected_mod_is_intro_skip(&ui.root, assets);
        sync_settings_panel(
            &mut ui.root,
            self.settings.snapshot(),
            show_settings,
            self.import_notice.current_and_tick(),
        );
        self.try_auto_continue(ui);
        self.try_auto_load_anyway(ui);
    }
}

impl IntroSkipExtension {
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
            match backup_latest_save() {
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

fn node_contains_id(node: &Node, id: &str) -> bool {
    node.id == id || node.child.iter().any(|child| node_contains_id(child, id))
}

fn find_node<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    if node.id == id {
        return Some(node);
    }
    node.child.iter().find_map(|child| find_node(child, id))
}

fn node_is_visible(node: &Node, id: &str) -> bool {
    (node.id == id && node.visible) || node.child.iter().any(|child| node_is_visible(child, id))
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

fn is_click_for(event: &UIEvent, first: &str, second: &str) -> bool {
    let matches_id = |value: &str| value.contains(first) || value.contains(second);
    match event {
        UIEvent::Click { item, path } => matches_id(item) || matches_id(path),
        UIEvent::CheckboxSelect { path, .. } => matches_id(path),
        _ => false,
    }
}

fn register_setting_handlers(
    ui: &mut GameUI,
    settings: Arc<RuntimeSettings>,
    import_notice: Arc<ImportNotice>,
) {
    let toggle_settings = Arc::clone(&settings);
    ui.filter_handler.push((
        Rc::new(|event| is_click_for(event, DISCLAIMER_ON_ID, DISCLAIMER_OFF_ID)),
        Rc::new(move |_context| {
            toggle_atomic(&toggle_settings.skip_disclaimer);
            save_runtime_settings(&toggle_settings);
        }),
    ));

    let toggle_settings = Arc::clone(&settings);
    ui.filter_handler.push((
        Rc::new(|event| is_click_for(event, CONTINUE_ON_ID, CONTINUE_OFF_ID)),
        Rc::new(move |_context| {
            toggle_atomic(&toggle_settings.auto_continue);
            save_runtime_settings(&toggle_settings);
        }),
    ));

    ui.filter_handler.push((
        Rc::new(|event| is_click_for(event, FORCE_ON_ID, FORCE_OFF_ID)),
        Rc::new(move |_context| {
            toggle_atomic(&settings.auto_load_anyway);
            save_runtime_settings(&settings);
        }),
    ));
    ui.filter_handler.push((
        Rc::new(|event| is_click_for(event, IMPORT_BACKUP_ID, IMPORT_BACKUP_ID)),
        Rc::new(move |_context| match import_latest_backup() {
            Ok(path) => {
                record_backup_status(&format!(
                    "Imported latest backup into the Load menu: {}",
                    path.display()
                ));
                import_notice.show(IMPORT_NOTICE_SUCCESS);
            }
            Err(error) => {
                record_backup_status(&format!("Backup import failed: {error}"));
                import_notice.show(IMPORT_NOTICE_FAILURE);
            }
        }),
    ));
}

fn toggle_atomic(value: &AtomicBool) {
    value.fetch_xor(true, Ordering::AcqRel);
}

fn selected_mod_is_intro_skip(root: &Node, assets: &Assets) -> bool {
    let Some(name_node) = find_node(root, "mod_name") else {
        return false;
    };
    if name_node.runner.type_name() != "engine_ui::runner::label::LabelRunner" {
        return false;
    }
    // NodeRunner does not expose a safe downcast helper in the 0.5.0 SDK.
    // The exact type-name guard keeps this cast limited to the known label runner.
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
    import_status: u8,
) {
    set_node_visible(root, SETTINGS_PANEL_ID, show_settings);
    set_node_visible(root, DISCLAIMER_ON_ID, settings.skip_disclaimer);
    set_node_visible(root, DISCLAIMER_OFF_ID, !settings.skip_disclaimer);
    set_node_visible(root, CONTINUE_ON_ID, settings.auto_continue);
    set_node_visible(root, CONTINUE_OFF_ID, !settings.auto_continue);
    set_node_visible(root, FORCE_ON_ID, settings.auto_load_anyway);
    set_node_visible(root, FORCE_OFF_ID, !settings.auto_load_anyway);
    set_node_visible(root, IMPORT_DEFAULT_ID, import_status == IMPORT_NOTICE_NONE);
    set_node_visible(
        root,
        IMPORT_SUCCESS_ID,
        import_status == IMPORT_NOTICE_SUCCESS,
    );
    set_node_visible(
        root,
        IMPORT_FAILURE_ID,
        import_status == IMPORT_NOTICE_FAILURE,
    );
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

fn prune_expired_backups(directory: &std::path::Path) -> Result<usize, String> {
    let now = SystemTime::now();
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
        if age <= BACKUP_RETENTION {
            continue;
        }
        fs::remove_file(&path)
            .map_err(|error| format!("cannot remove expired backup {}: {error}", path.display()))?;
        removed += 1;
    }
    Ok(removed)
}

fn cleanup_expired_backups() {
    let Some(backups) = backup_dir() else {
        return;
    };
    if !backups.exists() {
        return;
    }
    match prune_expired_backups(&backups) {
        Ok(removed) if removed > 0 => {
            record_backup_status(&format!("Removed {removed} backup(s) older than 7 days"));
        }
        Err(error) => {
            record_backup_status(&format!("Backup retention cleanup warning: {error}"));
        }
        _ => {}
    }
}

fn backup_latest_save() -> Result<PathBuf, String> {
    let data_dir = game_data_dir().ok_or("APPDATA is unavailable")?;
    let source = latest_save_in(&data_dir)?;
    let backups = backup_dir().ok_or("APPDATA is unavailable")?;
    fs::create_dir_all(&backups)
        .map_err(|error| format!("cannot create {}: {error}", backups.display()))?;
    cleanup_expired_backups();

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

fn import_latest_backup() -> Result<PathBuf, String> {
    let backups = backup_dir().ok_or("APPDATA is unavailable")?;
    let source = latest_save_in(&backups)?;
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
    let mut first_buffer = [0_u8; 64 * 1024];
    let mut second_buffer = [0_u8; 64 * 1024];

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
    let Some(path) =
        settings_path().and_then(|path| path.parent().map(|dir| dir.join("backup_status.log")))
    else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}: {message}", local_timestamp());
    }
}

fn settings_path() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let game_dir = executable.parent()?;
    Some(game_dir.join("mods").join(MOD_ID).join("settings.json"))
}

fn load_settings() -> Settings {
    let mut settings = Settings::default();
    let Some(path) = settings_path() else {
        return settings;
    };
    let Ok(source) = fs::read_to_string(path) else {
        return settings;
    };

    settings.skip_disclaimer =
        read_json_bool(&source, "skip_disclaimer").unwrap_or(settings.skip_disclaimer);
    settings.auto_continue =
        read_json_bool(&source, "auto_continue").unwrap_or(settings.auto_continue);
    settings.auto_load_anyway =
        read_json_bool(&source, "auto_load_anyway").unwrap_or(settings.auto_load_anyway);
    settings
}

fn save_runtime_settings(settings: &RuntimeSettings) {
    let Some(path) = settings_path() else {
        return;
    };
    let settings = settings.snapshot();
    let source = format!(
        "{{\n  \"skip_disclaimer\": {},\n  \"auto_continue\": {},\n  \"auto_load_anyway\": {}\n}}\n",
        settings.skip_disclaimer, settings.auto_continue, settings.auto_load_anyway
    );
    if let Err(error) = fs::write(path, source) {
        eprintln!("intro_skip: failed to save settings: {error}");
    }
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

#[repr(C)]
struct ClientRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
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
    fn GetClientRect(window: *mut c_void, rect: *mut ClientRect) -> i32;
    fn PostMessageW(window: *mut c_void, message: u32, wparam: usize, lparam: isize) -> i32;
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

fn post_ui_click(ui: &GameUI, ui_x: f32, ui_y: f32) -> bool {
    let window_title: Vec<u16> = "Teamfight Manager2"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let window = unsafe { FindWindowW(std::ptr::null(), window_title.as_ptr()) };
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

    unsafe {
        PostMessageW(window, WM_MOUSEMOVE, 0, position) != 0
            && PostMessageW(window, WM_LBUTTONDOWN, MK_LBUTTON, position) != 0
            && PostMessageW(window, WM_LBUTTONUP, 0, position) != 0
    }
}

fn init(_ctx: &GameCtx) -> ModRegistration {
    let settings = Arc::new(RuntimeSettings::new(load_settings()));
    let mut registration = ModRegistration::new(MOD_ID);
    registration.set_extension(IntroSkipExtension {
        settings,
        import_notice: Arc::new(ImportNotice::new()),
        title_ready_frames: AtomicUsize::new(0),
        auto_continue_sent: AtomicBool::new(false),
        mismatch_ready_frames: AtomicUsize::new(0),
        mismatch_click_sent: AtomicBool::new(false),
        mismatch_backup_ready: AtomicBool::new(false),
        mismatch_backup_failed: AtomicBool::new(false),
        retention_checked: AtomicBool::new(false),
        handlers_registered: AtomicBool::new(false),
    });
    registration
}

declare_mod!(init);
