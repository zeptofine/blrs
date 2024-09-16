use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::fetching::request_builder::{self, random_ua, SerialProxyOptions};

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "zeptofine", "blrs").unwrap());

pub fn ensure_config_folder_exists() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(PROJECT_DIRS.config_local_dir())
}

pub static DEFAULT_LIBRARY_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf().join("builds"));

pub static DEFAULT_REPOS_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf().join("remote-repos"));

/// 4 hours
pub static FETCH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 6);

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSPaths {
    pub library: PathBuf,
    pub remote_repos: PathBuf,
    pub daily: Option<PathBuf>,
    pub experimental_path: Option<PathBuf>,
    pub patch_path: Option<PathBuf>,
}

impl Default for BLRSPaths {
    fn default() -> Self {
        Self {
            library: DEFAULT_LIBRARY_FOLDER.clone(),
            remote_repos: DEFAULT_REPOS_FOLDER.clone(),
            daily: None,
            experimental_path: None,
            patch_path: None,
        }
    }
}

impl BLRSPaths {
    pub fn daily(&self) -> PathBuf {
        self.daily.clone().unwrap_or(self.library.join("daily"))
    }

    pub fn experimental(&self) -> PathBuf {
        self.experimental_path
            .clone()
            .unwrap_or(self.library.join("experimental"))
    }

    pub fn patch(&self) -> PathBuf {
        self.patch_path
            .clone()
            .unwrap_or(self.library.join("patch"))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSConfig {
    pub last_time_checked: Option<DateTime<Utc>>,
    pub paths: BLRSPaths,
    pub user_agent: String,
    pub proxy_options: SerialProxyOptions,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self {
            last_time_checked: Default::default(),
            paths: Default::default(),
            user_agent: random_ua(),
            proxy_options: Default::default(),
        }
    }
}

impl BLRSConfig {
    /// Returns the default Figment used to configure BLRS.
    /// If no config folder is specified, uses the BLRS default config directory.
    pub fn default_figment(config_folder: Option<&Path>) -> Figment {
        Figment::new()
            .merge(Serialized::defaults(BLRSConfig::default()))
            .merge(Toml::file(
                config_folder
                    .unwrap_or_else(|| PROJECT_DIRS.config_local_dir())
                    .join("config.toml"),
            ))
    }

    pub fn client_builder(&self) -> reqwest::ClientBuilder {
        request_builder::builder(&self.user_agent, self.proxy_options.clone().try_into().ok())
    }
}
