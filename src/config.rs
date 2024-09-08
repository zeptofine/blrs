use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("", "zeptofine", "blrs").unwrap());

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BLRSConfig {
    last_time_checked: DateTime<Utc>,
}

impl Default for BLRSConfig {
    fn default() -> Self {
        Self::default_figment().extract().unwrap_or_default()
    }
}

impl BLRSConfig {
    pub fn default_figment() -> Figment {
        Figment::new()
            .merge(Toml::file(
                PROJECT_DIRS.config_local_dir().join("config.toml"),
            ))
            .merge(Env::prefixed("BLRS_"))
    }
    // pub fn default_builder() -> ConfigBuilder<DefaultState> {
    //     Config::builder().add_source(config::File::new(
    // PROJECT_DIRS
    //     .config_local_dir()
    //     .join("config.toml")
    //     .to_str()
    //     .unwrap(),
    //         config::FileFormat::Toml,
    //     )).add_source(config::Environment::)
    // }
}
