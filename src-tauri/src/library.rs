use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Game not found: {0}")]
    NotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomGame {
    pub id: String,
    pub title: String,
    pub executable: PathBuf,
    pub cover_image: Option<PathBuf>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

impl CustomGame {
    pub fn new(
        title: impl Into<String>,
        executable: impl Into<PathBuf>,
        cover_image: Option<PathBuf>,
        tags: Vec<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            executable: executable.into(),
            cover_image,
            tags,
            notes,
        }
    }
}

/// Manages the collection of custom (non-Steam) games, persisted to a JSON file.
pub struct Library {
    path: PathBuf,
    games: Vec<CustomGame>,
}

impl Library {
    /// Loads the library from `path`, creating an empty one if the file doesn't exist.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, LibraryError> {
        let path = path.into();
        let games = if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let games: Vec<CustomGame> = serde_json::from_str(&contents)?;
            log::info!("Library loaded: {} game(s) from {:?}", games.len(), path);
            games
        } else {
            log::info!("No library file found at {:?}, starting empty", path);
            Vec::new()
        };
        Ok(Self { path, games })
    }

    pub fn games(&self) -> &[CustomGame] {
        &self.games
    }

    pub fn add(&mut self, game: CustomGame) -> Result<&CustomGame, LibraryError> {
        log::info!("Adding game to library: {:?} (id={})", game.title, game.id);
        self.games.push(game);
        self.persist()?;
        Ok(self.games.last().unwrap())
    }

    pub fn remove(&mut self, id: &str) -> Result<CustomGame, LibraryError> {
        let index = self
            .games
            .iter()
            .position(|g| g.id == id)
            .ok_or_else(|| LibraryError::NotFound(id.to_string()))?;
        let removed = self.games.remove(index);
        log::info!(
            "Removed game from library: {:?} (id={})",
            removed.title,
            removed.id
        );
        self.persist()?;
        Ok(removed)
    }

    pub fn update(&mut self, updated: CustomGame) -> Result<&CustomGame, LibraryError> {
        let index = self
            .games
            .iter()
            .position(|g| g.id == updated.id)
            .ok_or_else(|| LibraryError::NotFound(updated.id.clone()))?;
        self.games[index] = updated;
        self.persist()?;
        Ok(&self.games[index])
    }

    pub fn get(&self, id: &str) -> Option<&CustomGame> {
        self.games.iter().find(|g| g.id == id)
    }

    fn persist(&self) -> Result<(), LibraryError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.games)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_game(title: &str, exe: &str) -> CustomGame {
        CustomGame::new(title, exe, None, vec![], None)
    }

    fn temp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("game_library_test_{}.json", Uuid::new_v4()));
        p
    }

    // --- CustomGame construction ---

    #[test]
    fn new_game_has_unique_id() {
        let a = make_game("Game A", "/path/a");
        let b = make_game("Game B", "/path/b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_game_stores_fields() {
        let game = CustomGame::new(
            "Hollow Knight",
            "/games/hollow_knight",
            Some(PathBuf::from("/images/hk.png")),
            vec!["metroidvania".to_string()],
            Some("great game".to_string()),
        );
        assert_eq!(game.title, "Hollow Knight");
        assert_eq!(game.executable, PathBuf::from("/games/hollow_knight"));
        assert_eq!(game.cover_image, Some(PathBuf::from("/images/hk.png")));
        assert_eq!(game.tags, vec!["metroidvania"]);
        assert_eq!(game.notes, Some("great game".to_string()));
    }

    // --- Library load ---

    #[test]
    fn load_nonexistent_file_returns_empty_library() {
        let lib = Library::load("/tmp/definitely_does_not_exist_abc123.json").unwrap();
        assert!(lib.games().is_empty());
    }

    #[test]
    fn load_existing_file_deserializes_games() {
        let path = temp_path();
        let games = vec![make_game("Game A", "/a"), make_game("Game B", "/b")];
        std::fs::write(&path, serde_json::to_string(&games).unwrap()).unwrap();

        let lib = Library::load(&path).unwrap();
        assert_eq!(lib.games().len(), 2);
        assert_eq!(lib.games()[0].title, "Game A");

        std::fs::remove_file(path).ok();
    }

    // --- add ---

    #[test]
    fn add_game_persists_to_disk() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        lib.add(make_game("Celeste", "/games/celeste")).unwrap();

        let lib2 = Library::load(&path).unwrap();
        assert_eq!(lib2.games().len(), 1);
        assert_eq!(lib2.games()[0].title, "Celeste");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn add_multiple_games() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        lib.add(make_game("A", "/a")).unwrap();
        lib.add(make_game("B", "/b")).unwrap();
        lib.add(make_game("C", "/c")).unwrap();

        assert_eq!(lib.games().len(), 3);
        std::fs::remove_file(path).ok();
    }

    // --- remove ---

    #[test]
    fn remove_existing_game() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        let game = lib.add(make_game("Deleted", "/d")).unwrap().clone();
        lib.remove(&game.id).unwrap();

        assert!(lib.games().is_empty());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn remove_nonexistent_id_returns_error() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();
        let result = lib.remove("nonexistent-id");
        assert!(matches!(result, Err(LibraryError::NotFound(_))));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn remove_persists_deletion() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        let game = lib.add(make_game("ToDelete", "/td")).unwrap().clone();
        lib.remove(&game.id).unwrap();

        let lib2 = Library::load(&path).unwrap();
        assert!(lib2.games().is_empty());
        std::fs::remove_file(path).ok();
    }

    // --- update ---

    #[test]
    fn update_game_changes_fields() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        let mut game = lib.add(make_game("Old Title", "/exe")).unwrap().clone();
        game.title = "New Title".to_string();
        lib.update(game.clone()).unwrap();

        assert_eq!(lib.get(&game.id).unwrap().title, "New Title");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn update_nonexistent_id_returns_error() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        let mut ghost = make_game("Ghost", "/g");
        ghost.id = "does-not-exist".to_string();
        let result = lib.update(ghost);
        assert!(matches!(result, Err(LibraryError::NotFound(_))));
        std::fs::remove_file(path).ok();
    }

    // --- get ---

    #[test]
    fn get_returns_correct_game() {
        let path = temp_path();
        let mut lib = Library::load(&path).unwrap();

        let game = lib.add(make_game("Find Me", "/fm")).unwrap().clone();
        let found = lib.get(&game.id).unwrap();
        assert_eq!(found.title, "Find Me");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn get_returns_none_for_missing_id() {
        let path = temp_path();
        let lib = Library::load(&path).unwrap();
        assert!(lib.get("no-such-id").is_none());
        std::fs::remove_file(path).ok();
    }
}
