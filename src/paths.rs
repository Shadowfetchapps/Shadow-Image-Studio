use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub const APP_ID: &str = "com.shadowfetch.ImageStudio";
pub const APP_NAME: &str = "Shadow Image Studio";
pub const APP_ICON: &str = "shadow-image-studio";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_WEBSITE: &str = "https://github.com/Shadowfetchapps/Shadow-Image-Studio";

pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| {
        Error::user("Could not find the user configuration directory.")
    })?;
    Ok(base.join("shadow-image-studio"))
}

pub fn settings_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("settings.json"))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|err| {
        Error::detailed(format!("Could not create folder {}", path.display()), err.to_string())
    })
}

pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let ext = ext.trim_start_matches('.');
    let mut candidate = dir.join(format!("{stem}.{ext}"));
    if !candidate.exists() {
        return candidate;
    }
    for n in 2..10_000 {
        candidate = dir.join(format!("{stem} ({n}).{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem}-{}.{ext}", std::process::id()))
}

pub fn display_home_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
