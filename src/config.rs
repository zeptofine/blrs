use std::{path::PathBuf, sync::LazyLock};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "zeptofine", "blrs").unwrap());

pub static DEFAULT_LIBRARY_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf().join("builds"));

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSLibraryPaths {
    pub library: PathBuf,
    pub daily: Option<PathBuf>,
    pub experimental_path: Option<PathBuf>,
    pub patch_path: Option<PathBuf>,
}

impl Default for BLRSLibraryPaths {
    fn default() -> Self {
        Self {
            library: DEFAULT_LIBRARY_FOLDER.clone(),
            daily: None,
            experimental_path: None,
            patch_path: None,
        }
    }
}

impl BLRSLibraryPaths {
    fn daily(&self) -> PathBuf {
        self.daily.clone().unwrap_or(self.library.join("daily"))
    }

    fn experimental(&self) -> PathBuf {
        self.experimental_path
            .clone()
            .unwrap_or(self.library.join("experimental"))
    }

    fn patch(&self) -> PathBuf {
        self.patch_path
            .clone()
            .unwrap_or(self.library.join("patch"))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSConfig {
    pub last_time_checked: Option<DateTime<Utc>>,
    pub paths: BLRSLibraryPaths,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self {
            last_time_checked: None,
            paths: BLRSLibraryPaths::default(),
        }
    }
}

impl BLRSConfig {
    pub fn default_figment() -> Figment {
        Figment::new()
            .merge(Serialized::defaults(BLRSConfig::default()))
            .merge(Toml::file(
                PROJECT_DIRS.config_local_dir().join("config.toml"),
            ))
    }
}
