use serde::{Deserialize, Serialize};

use crate::document::ExportFormat;
use crate::error::Result;
use crate::paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Light,
            2 => Self::Dark,
            _ => Self::System,
        }
    }
    pub fn index(self) -> u32 {
        match self {
            Self::System => 0,
            Self::Light => 1,
            Self::Dark => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub export_format: u32,
    pub export_quality: u8,
    pub strip_metadata: bool,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            export_format: ExportFormat::Png.index(),
            export_quality: 88,
            strip_metadata: false,
            theme: Theme::System,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let Ok(path) = paths::settings_path() else {
            return Self::default();
        };
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = paths::settings_path()?;
        if let Some(parent) = path.parent() {
            paths::ensure_dir(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap())?;
        Ok(())
    }

    pub fn format(&self) -> ExportFormat {
        ExportFormat::from_index(self.export_format)
    }
}
