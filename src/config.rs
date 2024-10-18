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
use reqwest::Proxy;
use serde::{Deserialize, Serialize};

use crate::fetching::{
    authentication::GithubAuthentication,
    build_repository::{BuildRepo, DEFAULT_REPOS},
    request_builder::{random_ua, ProxyOptions, SerialProxyOptions},
};

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "zeptofine", "blrs").unwrap());

pub fn ensure_config_folder_exists() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(PROJECT_DIRS.config_local_dir())
}

/// Libraries should be structured like this:
///```txt
/// builds
/// |
/// +-<repo_id>
/// | |
/// | +-<individual_build_full_version>
/// | |  +-<n.n>
/// | |  +-blender.exe
/// | |  + ...
/// | +-<second_build>
/// | |
/// | +  ...
/// |
/// + ...
///```
pub static DEFAULT_LIBRARY_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf().join("builds"));

/// Repos should be structured like this:
///```md
/// remote-repos
/// |
/// +-<repo_id_0>.json
/// |
/// +-<repo_id_1>.json
/// |
/// +-<repo_id_2>.json
/// + ...
///```
pub static DEFAULT_REPOS_FOLDER: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf().join("remote-repos"));

/// 4 hours
pub static FETCH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 6);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSPaths {
    /// The path that holds all of the downloaded builds.
    pub library: PathBuf,
    /// The path that holds all of the repo cache .json files.
    pub remote_repos: PathBuf,
}

impl BLRSPaths {
    pub fn path_to_repo(&self, br: &BuildRepo) -> PathBuf {
        self.library.join(&br.repo_id)
    }
}

impl Default for BLRSPaths {
    fn default() -> Self {
        Self {
            library: DEFAULT_LIBRARY_FOLDER.clone(),
            remote_repos: DEFAULT_REPOS_FOLDER.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BLRSConfig {
    pub last_time_checked: Option<DateTime<Utc>>,
    pub paths: BLRSPaths,
    pub user_agent: String,
    pub repos: Vec<BuildRepo>,
    pub proxy_options: Option<SerialProxyOptions>,
    pub gh_auth: Option<GithubAuthentication>,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self {
            last_time_checked: Default::default(),
            paths: Default::default(),
            user_agent: random_ua(),
            repos: DEFAULT_REPOS.clone().into_iter().collect(),
            proxy_options: Default::default(),
            gh_auth: Default::default(),
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

    pub fn client_builder(&self, use_gh_auth: bool) -> reqwest::ClientBuilder {
        let user_agent: &str = &self.user_agent;
        let proxy: Option<ProxyOptions> = self
            .proxy_options
            .clone()
            .and_then(|opts| opts.try_into().ok());
        let mut r = reqwest::ClientBuilder::new().user_agent(user_agent);

        r = match (use_gh_auth, &self.gh_auth) {
            (true, Some(auth)) => {
                let mut auth_value = reqwest::header::HeaderValue::from_str(&format![
                    "{} {}",
                    auth.user, auth.token
                ])
                .unwrap();
                auth_value.set_sensitive(true);
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::AUTHORIZATION, auth_value);
                r.default_headers(headers)
            }
            _ => r,
        };

        r = match proxy {
            None => r,
            Some(options) => r.proxy(
                Proxy::all(options.url)
                    .unwrap()
                    .basic_auth(&options.user, &options.password),
            ),
        };

        r
    }
}
