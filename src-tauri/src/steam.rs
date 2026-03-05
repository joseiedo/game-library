use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum SteamError {
    #[error("Steam installation not found")]
    NotFound,
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ShortcutGame {
    pub app_id: u32,
    pub app_name: String,
    pub exe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteamGame {
    pub app_id: u64,
    pub name: String,
    pub install_dir: PathBuf,
    // Shortcuts are non-steam games or applications we've manually added
    // to Steam via Games > Add a Non-Steam Game. Steam assigns them a local
    // CRC-derived ID and tracks them in a binary file called shortcuts.vdf.
    // They don't have a real Steam App ID, so they launch via steam://rungameid/
    // instead of the usual steam://run/ URI.
    //
    // CRC: https://en.wikipedia.org/wiki/Cyclic_redundancy_check
    // Steam repurposes it here as a way to generate a unique ID for non-Steam games.
    // It takes the game's exe path and name, runs them through the CRC algorithm,
    // and the resulting number becomes the game's local ID. It's deterministic
    // (same exe + name always produces the same ID), but it's purely local,
    // Valve's servers have no knowledge of it, which is why these games can't use
    // the normal Steam store infrastructure.
    pub is_shortcut: bool,
}

#[allow(dead_code)]
impl SteamGame {
    pub fn launch_uri(&self) -> String {
        if self.is_shortcut {
            let full_id = (self.app_id << 32) | 0x02000000u64;
            format!("steam://rungameid/{}", full_id)
        } else {
            format!("steam://run/{}", self.app_id)
        }
    }
}

/// Finds all shortcuts.vdf files across all Steam user accounts.
pub fn find_shortcuts_vdf_paths(steam_root: &Path) -> Vec<PathBuf> {
    let userdata = steam_root.join("userdata");
    let mut results = Vec::new();

    let Ok(entries) = std::fs::read_dir(&userdata) else {
        return results;
    };

    for entry in entries.flatten() {
        let path = entry.path().join("config/shortcuts.vdf");
        if path.exists() {
            results.push(path);
        }
    }

    results
}

/// Parses a binary shortcuts.vdf file and returns the list of non-Steam shortcuts.
pub fn parse_shortcuts_vdf(data: &[u8]) -> Vec<ShortcutGame> {
    let mut games = Vec::new();
    let mut i = 0;

    // Binary VDF markers
    const TYPE_MAP: u8 = 0x00; // start of a map/object
    const TYPE_STRING: u8 = 0x01; // null-terminated string value
    const TYPE_INT32: u8 = 0x02; // 4-byte little-endian int value
    const END_MAP: u8 = 0x08; // end of map/object

    fn read_cstring(data: &[u8], pos: &mut usize) -> String {
        let start = *pos;
        while *pos < data.len() && data[*pos] != 0x00 {
            *pos += 1;
        }
        let s = String::from_utf8_lossy(&data[start..*pos]).to_string();
        *pos += 1; // consume null terminator
        s
    }

    fn read_u32_le(data: &[u8], pos: &mut usize) -> u32 {
        if *pos + 4 > data.len() {
            return 0;
        }
        let val = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
        *pos += 4;
        val
    }

    while i < data.len() {
        if data[i] != TYPE_MAP {
            i += 1;
            continue;
        }
        i += 1;

        let key = read_cstring(data, &mut i);
        // Only process numeric keys (the per-game entries)
        if key
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            let mut app_id: u32 = 0;
            let mut app_name = String::new();
            let mut exe = String::new();

            while i < data.len() && data[i] != END_MAP {
                let field_type = data[i];
                i += 1;
                let field_name = read_cstring(data, &mut i);

                match field_type {
                    TYPE_INT32 => {
                        let val = read_u32_le(data, &mut i);
                        if field_name.eq_ignore_ascii_case("appid") {
                            app_id = val;
                        }
                    }
                    TYPE_STRING => {
                        let val = read_cstring(data, &mut i);
                        match field_name.to_lowercase().as_str() {
                            "appname" => app_name = val,
                            "exe" => exe = val,
                            _ => {}
                        }
                    }
                    END_MAP => break,
                    _ => {
                        // Each game entry in shortcuts.vdf has multiple fields,
                        // not just appid, AppName, and exe.
                        // Steam stores a bunch of other stuff like:
                        //
                        // IsHidden (boolean, 1 byte),
                        // AllowDesktopConfig (boolean, 1 byte),
                        // LastPlayTime (uint64, 8 bytes),
                        // icon, tags, etc..
                        //
                        // Each field has a type byte before its name that tells
                        // us how many bytes the value takes. We only care about
                        // TYPE_INT32 (0x02) and TYPE_STRING (0x01), but when we
                        // hit any other type we need to skip past its value to
                        // get to the next field.
                        match field_type {
                            // 0x03 = single byte (boolean/uint8), skip 1 byte
                            // 0x04 = color, skip 4 bytes
                            // 0x05 = uint64, skip 8 bytes
                            // anything else we don't know, we advance one byte
                            0x03 => {
                                i += 1;
                            }
                            0x04 => {
                                i += 4;
                            }
                            0x05 => {
                                i += 8;
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }
                }
            }

            if !app_name.is_empty() {
                games.push(ShortcutGame {
                    app_id,
                    app_name,
                    exe,
                });
            }
        }
    }

    games
}

/// Discovers all non-Steam shortcut games for a given Steam root.
pub fn discover_shortcut_games(steam_root: &Path) -> Vec<ShortcutGame> {
    find_shortcuts_vdf_paths(steam_root)
        .iter()
        .flat_map(|path| {
            std::fs::read(path)
                .map(|data| parse_shortcuts_vdf(&data))
                .unwrap_or_default()
        })
        .collect()
}

/// Returns the default Steam root path for the current OS.
fn default_steam_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home).join("Library/Application Support/Steam"))
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        // Check both common locations
        let xdg = PathBuf::from(home.clone()).join(".steam/steam");
        if xdg.exists() {
            return Some(xdg);
        }
        Some(PathBuf::from(home).join(".local/share/Steam"))
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from("C:/Program Files (x86)/Steam"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Finds all Steam library folder paths by parsing `libraryfolders.vdf`.
pub fn find_library_paths(steam_root: &Path) -> Result<Vec<PathBuf>, SteamError> {
    let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");
    let contents = std::fs::read_to_string(&vdf_path)?;
    parse_library_paths_from_vdf(&contents, steam_root)
}

/// Parses library folder paths from the contents of `libraryfolders.vdf`.
/// The steam root's own `steamapps/` directory is always included.
pub fn parse_library_paths_from_vdf(
    vdf: &str,
    steam_root: &Path,
) -> Result<Vec<PathBuf>, SteamError> {
    let mut paths: Vec<PathBuf> = vec![steam_root.join("steamapps")];

    // Each library folder entry looks like:   "path"   "/some/path"
    for line in vdf.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            // Extract the value between the second pair of quotes
            if let Some(value) = extract_quoted_value(trimmed, 1) {
                let lib_path = PathBuf::from(value).join("steamapps");
                if !paths.contains(&lib_path) {
                    paths.push(lib_path);
                }
            }
        }
    }

    Ok(paths)
}

/// Reads all `appmanifest_*.acf` files in a steamapps directory and returns the games found.
pub fn read_games_from_library(steamapps_dir: &Path) -> Vec<SteamGame> {
    WalkDir::new(steamapps_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with("appmanifest_") && n.ends_with(".acf"))
                .unwrap_or(false)
        })
        .filter_map(|e| parse_acf_file(e.path()))
        .collect()
}

/// Parses a single `appmanifest_*.acf` file into a [`SteamGame`].
pub fn parse_acf_file(path: &Path) -> Option<SteamGame> {
    let contents = std::fs::read_to_string(path).ok()?;
    parse_acf(&contents, path.parent()?)
}

/// Parses the ACF content and constructs a [`SteamGame`].
pub fn parse_acf(contents: &str, steamapps_dir: &Path) -> Option<SteamGame> {
    let app_id = find_acf_value(contents, "appid")?.parse::<u64>().ok()?;
    let name = find_acf_value(contents, "name")?;
    let install_dir_name = find_acf_value(contents, "installdir")?;
    let install_dir = steamapps_dir.join("common").join(install_dir_name);

    Some(SteamGame {
        app_id,
        name,
        install_dir,
        is_shortcut: false,
    })
}

/// Discovers all installed Steam games on the system.
pub fn discover_games() -> Result<Vec<SteamGame>, SteamError> {
    let root = default_steam_root().ok_or(SteamError::NotFound)?;
    discover_games_at(&root)
}

/// Discovers all installed Steam games starting from a specific Steam root.
pub fn discover_games_at(steam_root: &Path) -> Result<Vec<SteamGame>, SteamError> {
    if !steam_root.exists() {
        return Err(SteamError::NotFound);
    }
    let library_paths = find_library_paths(steam_root)?;
    let mut seen = std::collections::HashSet::new();
    let games: Vec<SteamGame> = library_paths
        .iter()
        .flat_map(|dir| read_games_from_library(dir))
        .filter(|g| seen.insert(g.app_id))
        .collect();

    Ok(games)
}

// --- helpers ---

/// Extracts the nth (0-indexed) quoted string value from a line.
fn extract_quoted_value(line: &str, index: usize) -> Option<String> {
    let mut chars = line.chars().peekable();
    let mut found = 0;
    loop {
        // Find next opening quote
        chars.find(|&c| c == '"')?;
        // Collect until closing quote
        let value: String = chars.by_ref().take_while(|&c| c != '"').collect();
        if found == index {
            return Some(value);
        }
        found += 1;
    }
}

/// Finds the value for a key in an ACF file (simple key-value line: `"key"  "value"`).
fn find_acf_value(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("\"{}\"", key)) {
            return extract_quoted_value(trimmed, 1);
        }
    }
    None
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // --- extract_quoted_value ---

    #[test]
    fn extracts_first_quoted_value() {
        assert_eq!(
            extract_quoted_value(r#""path"   "/home/user/Steam""#, 0),
            Some("path".to_string())
        );
    }

    #[test]
    fn extracts_second_quoted_value() {
        assert_eq!(
            extract_quoted_value(r#""path"   "/home/user/Steam""#, 1),
            Some("/home/user/Steam".to_string())
        );
    }

    #[test]
    fn returns_none_when_index_out_of_bounds() {
        assert_eq!(extract_quoted_value(r#""only_one""#, 1), None);
    }

    // --- find_acf_value ---

    #[test]
    fn finds_appid_in_acf() {
        let acf = r#"
            "AppState"
            {
                "appid"     "440"
                "name"      "Team Fortress 2"
                "installdir"    "Team Fortress 2"
            }
        "#;
        assert_eq!(find_acf_value(acf, "appid"), Some("440".to_string()));
    }

    #[test]
    fn finds_name_in_acf() {
        let acf = r#"
            "AppState"
            {
                "appid"     "440"
                "name"      "Team Fortress 2"
                "installdir"    "Team Fortress 2"
            }
        "#;
        assert_eq!(
            find_acf_value(acf, "name"),
            Some("Team Fortress 2".to_string())
        );
    }

    #[test]
    fn returns_none_for_missing_key() {
        let acf = r#""appid" "440""#;
        assert_eq!(find_acf_value(acf, "missing"), None);
    }

    // --- parse_acf ---

    #[test]
    fn parses_complete_acf_entry() {
        let acf = r#"
            "AppState"
            {
                "appid"         "570"
                "name"          "Dota 2"
                "installdir"    "dota 2 beta"
            }
        "#;
        let steamapps = PathBuf::from("/fake/steamapps");
        let game = parse_acf(acf, &steamapps).expect("should parse");
        assert_eq!(game.app_id, 570);
        assert_eq!(game.name, "Dota 2");
        assert_eq!(
            game.install_dir,
            PathBuf::from("/fake/steamapps/common/dota 2 beta")
        );
    }

    #[test]
    fn returns_none_for_invalid_app_id() {
        let acf = r#""appid" "not_a_number" "name" "Broken""#;
        assert!(parse_acf(acf, Path::new("/fake")).is_none());
    }

    // --- parse_library_paths_from_vdf ---

    #[test]
    fn parses_single_extra_library() {
        let vdf = r#"
            "libraryfolders"
            {
                "0"
                {
                    "path"  "/mnt/games"
                }
            }
        "#;
        let root = PathBuf::from("/default/steam");
        let paths = parse_library_paths_from_vdf(vdf, &root).unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], root.join("steamapps"));
        assert_eq!(paths[1], PathBuf::from("/mnt/games/steamapps"));
    }

    #[test]
    fn returns_only_root_when_no_extra_libraries() {
        let vdf = r#""libraryfolders" { }"#;
        let root = PathBuf::from("/default/steam");
        let paths = parse_library_paths_from_vdf(vdf, &root).unwrap();
        assert_eq!(paths, vec![root.join("steamapps")]);
    }

    #[test]
    fn deduplicates_library_paths() {
        let root = PathBuf::from("/default/steam");
        // The root's own steamapps path appears as "path" entry too
        let vdf = format!(
            r#""libraryfolders" {{ "0" {{ "path" "{}" }} }}"#,
            root.to_str().unwrap()
        );
        let paths = parse_library_paths_from_vdf(&vdf, &root).unwrap();
        let unique: std::collections::HashSet<_> = paths.iter().collect();
        assert_eq!(paths.len(), unique.len(), "paths should be deduplicated");
    }

    // --- SteamGame helpers ---

    #[test]
    fn launch_uri_format() {
        let game = SteamGame {
            app_id: 440,
            name: "Team Fortress 2".to_string(),
            install_dir: PathBuf::from("/fake"),
            is_shortcut: false,
        };
        assert_eq!(game.launch_uri(), "steam://run/440");
    }

    #[test]
    fn launch_uri_format_shortcut() {
        let game = SteamGame {
            app_id: 440,
            name: "Team Fortress 2".to_string(),
            install_dir: PathBuf::from("/fake"),
            is_shortcut: true,
        };
        let expected_id = (440u64 << 32) | 0x02000000u64;
        assert_eq!(game.launch_uri(), format!("steam://rungameid/{}", expected_id));
    }
}
