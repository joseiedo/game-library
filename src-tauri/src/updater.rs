use tauri::AppHandle;

/// Downloads a release asset to the user's Downloads directory.
/// Returns the absolute path to the saved file.
#[tauri::command]
pub async fn download_update(app: AppHandle, asset_url: String) -> Result<String, String> {
    let download_dir = app.path().download_dir().map_err(|e| e.to_string())?;

    let filename = asset_url
        .rsplit('/')
        .next()
        .ok_or_else(|| "Invalid asset URL".to_string())?
        .to_string();

    let dest = download_dir.join(&filename);

    let client = reqwest::Client::builder()
        .user_agent("game-library-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&asset_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {e}"))?;

    std::fs::write(&dest, &bytes).map_err(|e| format!("Write failed: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}
