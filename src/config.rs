use std::{path::Path, path::PathBuf, sync::LazyLock, time::Duration};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::fetching::{
    authentication::GithubAuthentication,
    build_repository::{BuildRepo, DEFAULT_REPOS},
    random_ua, SerialProxyOptions,
};

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};

/// This static variable holds the project's directory structure.
pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "zeptofine", "blrs").unwrap());

/// Ensures that the config folder exists for BLRS configuration files.
pub fn ensure_config_folder_exists() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(PROJECT_DIRS.config_local_dir())
}

/// The structure of the library folder where downloaded builds are stored.
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

/// The structure of the remote repos folder where repo cache .json files are stored.
///```txt
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

/// The interval at which to check for build repo updates (6 hours).
pub static FETCH_INTERVAL: Duration = Duration::from_secs(60 * 60 * 6);

/// Defines the paths where BLRS data is stored.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSPaths {
    /// The path that holds all of the downloaded builds.
    pub library: PathBuf,
    /// The path that holds all of the repo cache .json files.
    pub remote_repos: PathBuf,
}

impl BLRSPaths {
    /// Returns the path to a specific repository based on its ID.
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

/// Stores information about the last build launched and when the build repos were last checked.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct History {
    /// The last build that was launched.
    pub last_launched_build: Option<PathBuf>,
    /// The last time the build repos were checked for updates.
    pub last_time_checked: Option<DateTime<Utc>>,
}

// TODO: Encrypt the proxy options and the github authentication

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
    /// Contains options about setting a proxy for request making.
    proxy_options: Option<SerialProxyOptions>,
    /// Authentication details for GitHub
    gh_auth: Option<GithubAuthentication>,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self {
            user_agent: random_ua(),
            paths: Default::default(),
            repos: DEFAULT_REPOS.clone().into_iter().collect(),
            history: Default::default(),
            proxy_options: Default::default(),
            gh_auth: Default::default(),
        }
    }
}

impl BLRSConfig {
    /// Returns the default Figment used to configure BLRS.
    /// If no config folder is specified, uses the BLRS default config directory.
    #[cfg(feature = "figment")]
    #[cfg_attr(docsrs, doc(cfg(feature = "figment")))]
    pub fn default_figment(config_folder: Option<&Path>) -> Figment {
        Figment::new()
            .merge(Serialized::defaults(BLRSConfig::default()))
            .merge(Toml::file(
                config_folder
                    .unwrap_or_else(|| PROJECT_DIRS.config_local_dir())
                    .join("config.toml"),
            ))
    }

    /// A public method for updating the proxy options.
    pub fn update_proxy_options(&mut self, po: Option<SerialProxyOptions>) {
        self.proxy_options = po;
    }
    /// A public method for updating the github authentication.
    pub fn update_github_authentication(&mut self, ga: Option<GithubAuthentication>) {
        self.gh_auth = ga
    }

    /// Creates a ClientBuilder with the configured proxy options.
    #[cfg(feature = "reqwest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
    pub fn client_builder(&self, use_gh_auth: bool) -> reqwest::ClientBuilder {
        use reqwest::Proxy;

        use crate::fetching::ProxyOptions;

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
