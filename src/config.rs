use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    fetching::{
        build_repository::{BuildRepo, DEFAULT_REPOS},
        random_ua,
    },
    BLRSPaths, PROJECT_DIRS,
};

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

/// Stores information about the last build launched and when the build repos were last checked.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct History {
    /// The last build that was launched.
    pub last_launched_build: Option<PathBuf>,
    /// The last time the build repos were checked for updates.
    pub last_time_checked: Option<DateTime<Utc>>,
}

// TODO: Encrypt the github authentication somehow

///  Represents the main configuration struct for BLRS.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BLRSConfig {
    /// The user agent string used by BLRS when making network requests.
    pub user_agent: String,
    /// Defines paths for BLRS data storage.
    pub paths: BLRSPaths,
    /// A list of BuildRepo structs defining the available build repositories.
    pub repos: Vec<BuildRepo>,
    /// Contains information about the last launched build and repo update checks.
    pub history: History,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self {
            user_agent: random_ua(),
            paths: Default::default(),
            repos: DEFAULT_REPOS.clone().into_iter().collect(),
            history: Default::default(),
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

    /// Creates a ClientBuilder with the configured auth options.
    #[cfg(feature = "reqwest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
    pub fn client_builder(&self) -> reqwest::ClientBuilder {
        let user_agent: &str = &self.user_agent;

        reqwest::ClientBuilder::new().user_agent(user_agent)
    }
}
