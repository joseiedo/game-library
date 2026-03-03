mod epic;
mod fs_explorer;
mod launcher;
mod library;
mod settings;
mod steam;
mod updater;

use epic::EpicGame;
use launcher::LaunchTarget;
use library::{CustomGame, Library};
use settings::Settings;
use std::path::PathBuf;
use std::sync::Mutex;
use steam::SteamGame;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

struct AppState {
    library: Mutex<Library>,
    settings: Mutex<Settings>,
}

fn library_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("could not resolve app data dir")
        .join("custom_games.json")
}

fn settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("could not resolve app data dir")
        .join("settings.json")
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_steam_games() -> Result<Vec<SteamGame>, String> {
    let steam_root =
        PathBuf::from(std::env::var("HOME").unwrap_or_default() + "/.local/share/Steam");

    let shortcuts: Vec<SteamGame> = steam::discover_shortcut_games(&steam_root)
        .into_iter()
        .map(|s| SteamGame {
            app_id: s.app_id as u64,
            name: s.app_name,
            install_dir: PathBuf::from(&s.exe),
            is_shortcut: true,
        })
        .collect();

    match steam::discover_games() {
        Ok(mut games) => {
            games.extend(shortcuts);
            log::info!(
                "Steam discovery: found {} games (incl. shortcuts)",
                games.len()
            );
            Ok(games)
        }
        Err(e) => {
            log::warn!("Steam discovery failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_epic_games() -> Result<Vec<EpicGame>, String> {
    match epic::discover_games() {
        Ok(games) => {
            log::info!("Epic discovery: found {} games", games.len());
            Ok(games)
        }
        Err(e) => {
            log::warn!("Epic discovery failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_custom_games(state: State<AppState>) -> Vec<CustomGame> {
    state.library.lock().unwrap().games().to_vec()
}

#[tauri::command]
fn add_game(
    state: State<AppState>,
    title: String,
    executable: String,
    cover_image: Option<String>,
    tags: Vec<String>,
    notes: Option<String>,
) -> Result<CustomGame, String> {
    log::info!(
        "Adding custom game: title={:?} executable={:?}",
        title,
        executable
    );
    let game = CustomGame::new(
        title,
        executable,
        cover_image.map(PathBuf::from),
        tags,
        notes,
    );
    state
        .library
        .lock()
        .unwrap()
        .add(game)
        .map(|g| g.clone())
        .map_err(|e| {
            log::error!("Failed to add game: {}", e);
            e.to_string()
        })
}

#[tauri::command]
fn remove_game(state: State<AppState>, id: String) -> Result<(), String> {
    log::info!("Removing custom game: id={}", id);
    state
        .library
        .lock()
        .unwrap()
        .remove(&id)
        .map(|removed| {
            log::info!("Removed game: {:?} (id={})", removed.title, removed.id);
        })
        .map_err(|e| {
            log::error!("Failed to remove game id={}: {}", id, e);
            e.to_string()
        })
}

#[tauri::command]
fn edit_game(
    state: State<AppState>,
    id: String,
    title: String,
    executable: String,
    cover_image: Option<String>,
    tags: Vec<String>,
    notes: Option<String>,
) -> Result<CustomGame, String> {
    log::info!("Editing custom game: id={} title={:?}", id, title);
    let updated = CustomGame {
        id,
        title,
        executable: PathBuf::from(executable),
        cover_image: cover_image.map(PathBuf::from),
        tags,
        notes,
    };
    state
        .library
        .lock()
        .unwrap()
        .update(updated)
        .map(|g| g.clone())
        .map_err(|e| {
            log::error!("Failed to edit game: {}", e);
            e.to_string()
        })
}

#[tauri::command]
fn launch_game(
    _state: State<AppState>,
    _key: String,
    app_id: Option<u32>,
    is_shortcut: Option<bool>,
    executable: Option<String>,
    epic_launch_uri: Option<String>,
) -> Result<(), String> {
    log::info!(
        "launch_game: key={:?} app_id={:?} is_shortcut={:?} executable={:?} epic={:?}",
        _key,
        app_id,
        is_shortcut,
        executable,
        epic_launch_uri,
    );
    let target = match (app_id, epic_launch_uri, executable) {
        // (Some(id), _, _) => LaunchTarget::steam(id),
        (Some(id), _, _) => {
            if is_shortcut.unwrap_or(false) {
                LaunchTarget::steam_shortcut(id)
            } else {
                LaunchTarget::steam(id)
            }
        }
        (_, Some(uri), _) => LaunchTarget::epic_game(uri),
        (_, _, Some(path)) => LaunchTarget::executable(path),
        (None, None, None) => {
            log::warn!("launch_game called with no launch target");
            return Err("No launch target specified".to_string());
        }
    };
    launcher::launch(&target).map_err(|e| {
        log::error!("Launch failed for {:?}: {}", _key, e);
        e.to_string()
    })
}

// ---------------------------------------------------------------------------
// Ignored games commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_ignored_games(state: State<AppState>) -> Vec<String> {
    state
        .settings
        .lock()
        .unwrap()
        .ignored_game_keys()
        .iter()
        .cloned()
        .collect()
}

#[tauri::command]
fn add_ignored_game(state: State<AppState>, key: String) -> Result<(), String> {
    log::info!("Ignoring game: {}", key);
    state
        .settings
        .lock()
        .unwrap()
        .add_ignored(key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_ignored_game(state: State<AppState>, key: String) -> Result<(), String> {
    log::info!("Unignoring game: {}", key);
    state
        .settings
        .lock()
        .unwrap()
        .remove_ignored(&key)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Autostart commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_autostart(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let al = app.autolaunch();
    if enabled {
        al.enable().map_err(|e| e.to_string())
    } else {
        al.disable().map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// File-explorer commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn list_directory(path: String) -> Result<Vec<fs_explorer::DirEntry>, String> {
    fs_explorer::read_dir(&path)
}

#[tauri::command]
fn get_file_explorer_bookmarks() -> Vec<fs_explorer::Bookmark> {
    fs_explorer::get_bookmarks()
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    },
                ))
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let lib_path = library_path(app.handle());
            log::info!("Loading custom game library from {:?}", lib_path);
            let library = Library::load(lib_path).expect("failed to load game library");
            log::info!("Library ready: {} custom game(s)", library.games().len());

            let set_path = settings_path(app.handle());
            log::info!("Loading app settings from {:?}", set_path);
            let app_settings = Settings::load(set_path).expect("failed to load settings");
            log::info!(
                "Settings ready: {} ignored game(s)",
                app_settings.ignored_game_keys().len()
            );

            app.manage(AppState {
                library: Mutex::new(library),
                settings: Mutex::new(app_settings),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_steam_games,
            get_epic_games,
            get_custom_games,
            add_game,
            edit_game,
            remove_game,
            launch_game,
            list_directory,
            get_file_explorer_bookmarks,
            get_autostart,
            set_autostart,
            get_ignored_games,
            add_ignored_game,
            remove_ignored_game,
            updater::download_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
