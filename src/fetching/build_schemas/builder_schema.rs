use std::fmt::Debug;

use chrono::DateTime;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

use crate::{
    info::{build_info::VerboseVersion, parse_blender_ver},
    BasicBuildInfo, RemoteBuild,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct BlenderBuildSchema {
    pub app: String,
    pub url: String,
    pub version: String,
    pub branch: String,
    pub patch: Option<String>,
    pub hash: String,
    pub platform: String,
    pub architecture: String,
    pub file_mtime: usize,
    pub file_name: String,
    pub file_size: usize,
    pub file_extension: String,
    pub release_cycle: String, // stable,alpha,etc.
}

impl From<BlenderBuildSchema> for RemoteBuild {
    fn from(val: BlenderBuildSchema) -> Self {
        RemoteBuild {
            link: val.url.clone(),
            basic: BasicBuildInfo {
                ver: VerboseVersion::from(val.full_version()),
                commit_dt: DateTime::from_timestamp(val.file_mtime as i64, 0).unwrap(),
            },
        }
    }
}

impl BlenderBuildSchema {
    pub fn full_version(&self) -> Version {
        Version {
            pre: Prerelease::new(&self.release_cycle).unwrap(),
            build: BuildMetadata::new(&format!["{}.{}", self.branch, self.hash]).unwrap(),
            ..parse_blender_ver(&self.version, false).unwrap()
        }
    }
    pub fn full_version_and_platform(&self) -> Version {
        Version {
            pre: Prerelease::new(&format!["{}-{}", self.platform, self.release_cycle]).unwrap(),
            build: BuildMetadata::new(&format!["{}.{}", self.branch, self.hash]).unwrap(),
            ..parse_blender_ver(&self.version, false).unwrap()
        }
    }
}
