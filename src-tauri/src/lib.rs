mod epic;
mod fs_explorer;
mod launcher;
mod library;
mod steam;

use epic::EpicGame;
use launcher::LaunchTarget;
use library::{CustomGame, Library};
use std::path::PathBuf;
use std::sync::Mutex;
use steam::SteamGame;
use tauri::{AppHandle, Manager, State};

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

struct AppState {
    library: Mutex<Library>,
}

fn library_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("could not resolve app data dir")
        .join("custom_games.json")
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
        .setup(|app| {
            let path = library_path(app.handle());
            log::info!("Loading custom game library from {:?}", path);
            let library = Library::load(path).expect("failed to load game library");
            log::info!("Library ready: {} custom game(s)", library.games().len());
            app.manage(AppState {
                library: Mutex::new(library),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
