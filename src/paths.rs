use std::{path::PathBuf, sync::LazyLock, time::Duration};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::fetching::build_repository::BuildRepo;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BLRSPaths {
    /// The path that holds all of the downloaded builds.
    pub library: PathBuf,
    /// The path that holds all of the repo cache .json files.
    pub remote_repos: PathBuf,
}

impl AsRef<BLRSPaths> for BLRSPaths {
    fn as_ref(&self) -> &BLRSPaths {
        self
    }
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
