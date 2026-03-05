use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Executable not found: {0}")]
    ExecutableNotFound(String),
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchTarget {
    Steam { app_id: u32 },
    SteamShortcut { app_id: u32 },
    EpicGame { launch_uri: String },
    Executable { path: String },
}

impl LaunchTarget {
    pub fn steam(app_id: u32) -> Self {
        Self::Steam { app_id }
    }

    pub fn steam_shortcut(app_id: u32) -> Self {
        Self::SteamShortcut { app_id }
    }

    pub fn epic_game(launch_uri: impl Into<String>) -> Self {
        Self::EpicGame {
            launch_uri: launch_uri.into(),
        }
    }

    pub fn executable(path: impl Into<String>) -> Self {
        Self::Executable { path: path.into() }
    }

}

/// Launches the given target. For Steam and Epic games this opens the appropriate URI;
/// for custom games it delegates to [`spawn_executable`] (child is discarded).
pub fn launch(target: &LaunchTarget) -> Result<(), LaunchError> {
    match target {
        LaunchTarget::Steam { app_id } => launch_steam(*app_id),
        LaunchTarget::SteamShortcut { app_id } => {
            let full_id = ((*app_id as u64) << 32) | 0x02000000u64;
            let uri = format!("steam://rungameid/{}", full_id);
            open_uri(&uri)
        }
        LaunchTarget::EpicGame { launch_uri } => open_uri(launch_uri),
        LaunchTarget::Executable { path } => {
            spawn_executable(path)?;
            Ok(())
        }
    }
}

/// Opens the Steam URI for the given app ID using the OS default handler.
pub fn launch_steam(app_id: u32) -> Result<(), LaunchError> {
    let uri = format!("steam://run/{}", app_id);
    log::info!("Launching Steam game: app_id={} uri={}", app_id, uri);
    open_uri(&uri)
}

/// Spawns the game at `path` and returns the child process handle when available.
///
/// On macOS, if `path` is a `.app` bundle directory the system `open` command is used
/// — which hands off to launchd — so no direct child handle is returned (`None`).
/// On all other platforms, or when `path` points to a regular executable, the process
/// is spawned directly and `Some(child)` is returned.
pub fn spawn_executable(path: &str) -> Result<Option<std::process::Child>, LaunchError> {
    if !Path::new(path).exists() {
        log::warn!("Executable not found: {}", path);
        return Err(LaunchError::ExecutableNotFound(path.to_string()));
    }

    #[cfg(target_os = "macos")]
    if Path::new(path).is_dir() && path.ends_with(".app") {
        log::info!("Launching macOS app bundle via open: {}", path);
        Command::new("open").arg(path).spawn()?;
        return Ok(None);
    }

    log::info!("Spawning executable: {}", path);
    Ok(Some(Command::new(path).spawn()?))
}

/// Opens a URI using the platform's default handler.
fn open_uri(uri: &str) -> Result<(), LaunchError> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(uri).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(uri).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", uri]).spawn()?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = uri;
    }
    Ok(())
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    // --- EpicGame launch target ---

    #[test]
    fn epic_game_target_stores_uri() {
        let uri = "com.epicgames.launcher://apps/ns%3Aid%3AFortnite?action=launch&silent=true";
        let target = LaunchTarget::epic_game(uri);
        assert_eq!(
            target,
            LaunchTarget::EpicGame {
                launch_uri: uri.to_string()
            }
        );
    }

    // --- LaunchTarget ---

    #[test]
    fn executable_regression_target_builds() {
        let target = LaunchTarget::executable("/games/game.exe");
        assert_eq!(
            target,
            LaunchTarget::Executable {
                path: "/games/game.exe".to_string()
            }
        );
    }

    #[test]
    fn executable_target_stores_path() {
        let target = LaunchTarget::executable("/games/hollow_knight");
        assert_eq!(
            target,
            LaunchTarget::Executable {
                path: "/games/hollow_knight".to_string()
            }
        );
    }

    #[test]
    fn steam_target_stores_app_id() {
        let target = LaunchTarget::steam(570);
        assert_eq!(target, LaunchTarget::Steam { app_id: 570 });
    }

    // --- spawn_executable ---

    #[test]
    fn spawn_nonexistent_returns_error() {
        let err = spawn_executable("/absolutely/does/not/exist.exe").unwrap_err();
        assert!(matches!(err, LaunchError::ExecutableNotFound(_)));
    }

    #[cfg(unix)]
    #[test]
    fn spawn_binary_returns_some_child() {
        if !Path::new("/usr/bin/true").exists() {
            return;
        }
        let child = spawn_executable("/usr/bin/true")
            .expect("should not error")
            .expect("direct binary should give Some(child)");
        drop(child); // let it clean up
    }

    #[cfg(unix)]
    #[test]
    fn child_is_running_then_killed() {
        let mut child = std::process::Command::new("/bin/sleep")
            .arg("60")
            .spawn()
            .expect("spawn failed");

        assert!(child.try_wait().unwrap().is_none(), "should be running");

        child.kill().expect("kill failed");
        let status = child.wait().expect("wait failed");
        assert!(!status.success());
    }

}
