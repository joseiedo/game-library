use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpicGame {
    /// Stable unique identifier from the manifest `AppName` field.
    pub app_name: String,
    /// Human-readable title from the manifest `DisplayName` field.
    pub display_name: String,
    /// Absolute path to the game's installation directory.
    pub install_location: PathBuf,
    /// Epic catalog namespace (used in the launch URI).
    pub catalog_namespace: String,
    /// Epic catalog item ID (used in the launch URI).
    pub catalog_item_id: String,
    /// Absolute path to a local cover image, or `None` when not found.
    pub cover_image: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum EpicError {
    #[error("Failed to read Epic manifest directory: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Manifest schema (only fields we care about)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Manifest {
    app_name: Option<String>,
    display_name: Option<String>,
    install_location: Option<String>,
    catalog_namespace: Option<String>,
    catalog_item_id: Option<String>,
    #[serde(rename = "bIsApplication", default)]
    b_is_application: bool,
    #[serde(rename = "bIsExecutable", default)]
    b_is_executable: bool,
    #[serde(rename = "bIsIncompleteInstall", default)]
    b_is_incomplete_install: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns all installed Epic games, or `Ok(vec![])` if the launcher is absent.
pub fn discover_games() -> Result<Vec<EpicGame>, EpicError> {
    match manifest_dir() {
        Some(dir) => discover_games_from(&dir),
        None => Ok(vec![]),
    }
}

/// Discovers Epic games from a specific manifest directory (used in tests).
pub fn discover_games_from(manifest_dir: &Path) -> Result<Vec<EpicGame>, EpicError> {
    if !manifest_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(manifest_dir)?;
    let mut games = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("item") {
            if let Some(game) = parse_manifest(&path) {
                games.push(game);
            }
        }
    }

    Ok(games)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns the platform-appropriate Epic manifest directory path.
fn manifest_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(
            PathBuf::from(home)
                .join("Library/Application Support/Epic/EpicGamesLauncher/Data/Manifests"),
        )
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(
            r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests",
        ))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Parses a single `.item` manifest file; returns `None` if it should be skipped.
fn parse_manifest(path: &Path) -> Option<EpicGame> {
    let contents = std::fs::read_to_string(path).ok()?;
    let m: Manifest = serde_json::from_str(&contents).ok()?;

    // Apply Epic filter rules
    if !m.b_is_application || !m.b_is_executable || m.b_is_incomplete_install {
        return None;
    }

    let app_name = m.app_name.filter(|s| !s.is_empty())?;
    let display_name = m.display_name.filter(|s| !s.is_empty())?;
    let install_location = m.install_location.filter(|s| !s.is_empty())?;
    let catalog_namespace = m.catalog_namespace.unwrap_or_default();
    let catalog_item_id = m.catalog_item_id.unwrap_or_default();

    let install_path = PathBuf::from(&install_location);
    let cover_image = find_cover_image(&install_path);

    Some(EpicGame {
        app_name,
        display_name,
        install_location: install_path,
        catalog_namespace,
        catalog_item_id,
        cover_image,
    })
}

/// Scans the game's install directory (depth 1) for the first PNG or JPEG file.
fn find_cover_image(install_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(install_dir).ok()?;
    entries.flatten().find_map(|e| {
        let p = e.path();
        if p.is_file() {
            match p.extension().and_then(|ex| ex.to_str()) {
                Some("png") | Some("jpg") | Some("jpeg") => Some(p),
                _ => None,
            }
        } else {
            None
        }
    })
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Writes a minimal valid .item manifest to `dir/name.item`.
    fn write_manifest(dir: &Path, name: &str, extra: &str) {
        let content = format!(
            r#"{{
  "AppName": "{name}",
  "DisplayName": "{name} Display",
  "InstallLocation": "{dir_str}",
  "CatalogNamespace": "ns-{name}",
  "CatalogItemId": "id-{name}",
  "bIsApplication": true,
  "bIsExecutable": true,
  "bIsIncompleteInstall": false
  {extra}
}}"#,
            name = name,
            dir_str = dir.to_string_lossy().replace('\\', "/"),
            extra = extra,
        );
        fs::write(dir.join(format!("{name}.item")), content).unwrap();
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("epic_test_{}_{}", label, std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // ------------------------------------------------------------------ T005
    #[test]
    fn happy_path() {
        let manifest_dir = make_temp_dir("happy");
        write_manifest(&manifest_dir, "GameA", "");
        write_manifest(&manifest_dir, "GameB", "");

        let games = discover_games_from(&manifest_dir).expect("should succeed");
        assert_eq!(games.len(), 2);

        let mut names: Vec<String> = games.iter().map(|g| g.app_name.clone()).collect();
        names.sort();
        assert_eq!(names, ["GameA", "GameB"]);
        // Every game should have display_name and install_location populated
        assert!(games.iter().all(|g| !g.display_name.is_empty()));
        assert!(games.iter().all(|g| !g.catalog_namespace.is_empty()));

        fs::remove_dir_all(&manifest_dir).ok();
    }

    // ------------------------------------------------------------------ T006
    #[test]
    fn launcher_not_installed() {
        let missing = std::env::temp_dir().join("epic_test_absent_dir_99999");
        // Ensure it does not exist
        let _ = fs::remove_dir_all(&missing);

        let games = discover_games_from(&missing).expect("absent dir should return Ok");
        assert!(games.is_empty());
    }

    // ------------------------------------------------------------------ T007
    #[test]
    fn malformed_manifest_skipped() {
        let manifest_dir = make_temp_dir("malformed");
        write_manifest(&manifest_dir, "GoodGame", "");
        fs::write(manifest_dir.join("bad.item"), b"not valid json at all {{{{").unwrap();

        let games = discover_games_from(&manifest_dir).expect("should succeed despite bad file");
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_name, "GoodGame");

        fs::remove_dir_all(&manifest_dir).ok();
    }

    // ------------------------------------------------------------------ T008
    #[test]
    fn incomplete_install_excluded() {
        let manifest_dir = make_temp_dir("incomplete");
        // bIsIncompleteInstall overrides the valid fields
        let content = format!(
            r#"{{
  "AppName": "IncompleteGame",
  "DisplayName": "Incomplete Game",
  "InstallLocation": "{}",
  "CatalogNamespace": "ns",
  "CatalogItemId": "id",
  "bIsApplication": true,
  "bIsExecutable": true,
  "bIsIncompleteInstall": true
}}"#,
            manifest_dir.to_string_lossy().replace('\\', "/")
        );
        fs::write(manifest_dir.join("incomplete.item"), content).unwrap();

        let games = discover_games_from(&manifest_dir).expect("should succeed");
        assert!(games.is_empty(), "incomplete installs must be excluded");

        fs::remove_dir_all(&manifest_dir).ok();
    }

    // ------------------------------------------------------------------ T009
    #[test]
    fn non_application_excluded() {
        let manifest_dir = make_temp_dir("non_app");
        let content = format!(
            r#"{{
  "AppName": "NotAnApp",
  "DisplayName": "Not An App",
  "InstallLocation": "{}",
  "CatalogNamespace": "ns",
  "CatalogItemId": "id",
  "bIsApplication": false,
  "bIsExecutable": true,
  "bIsIncompleteInstall": false
}}"#,
            manifest_dir.to_string_lossy().replace('\\', "/")
        );
        fs::write(manifest_dir.join("nonapp.item"), content).unwrap();

        let games = discover_games_from(&manifest_dir).expect("should succeed");
        assert!(games.is_empty(), "non-application entries must be excluded");

        fs::remove_dir_all(&manifest_dir).ok();
    }

}
