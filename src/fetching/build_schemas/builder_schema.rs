use std::fmt::Debug;

use chrono::DateTime;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

use crate::{
    info::{parse_blender_ver, VerboseVersion},
    BasicBuildInfo, RemoteBuild,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
/// Represents the schema of a Blender build. This is used in fetching builds from the official builder repos.
pub struct BlenderBuildSchema {
    /// The name of the application (usually "Blender").
    pub app: String,

    /// The URL to download the build.
    pub url: String,

    /// The version string of the Blender build.
    pub version: String,

    /// The Git branch this build was created from.
    pub branch: String,

    /// Optional patch version information.
    pub patch: Option<String>,

    /// The commit hash associated with this build.
    pub hash: String,

    /// The platform the build is for (e.g., "windows", "linux").
    pub platform: String,

    /// The architecture of the build (e.g., "x86_64").
    pub architecture: String,

    /// The last modification time of the build file in seconds since epoch.
    pub file_mtime: usize,

    /// The name of the build file without extension.
    pub file_name: String,

    /// The size of the build file in bytes.
    pub file_size: usize,

    /// The file extension of the build (e.g., "zip", "tar.xz").
    pub file_extension: String,

    /// The release cycle of the build (e.g., "stable", "alpha").
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
            platform: Some(val.platform),
            architecture: Some(val.architecture),
            file_extension: Some(val.file_extension),
        }
    }
}

impl BlenderBuildSchema {
    /// Constructs a `Version` object from the build schema's information.
    pub fn full_version(&self) -> Version {
        Version {
            pre: Prerelease::new(&self.release_cycle).unwrap(),
            build: BuildMetadata::new(&format!["{}.{}", self.branch, self.hash]).unwrap(),
            ..parse_blender_ver(&self.version, false).unwrap()
        }
    }

    /// Constructs a `Version` object from the build schema's information, including the platform in the prerelease.
    pub fn full_version_and_platform(&self) -> Version {
        Version {
            pre: Prerelease::new(&format!["{}-{}", self.platform, self.release_cycle]).unwrap(),
            build: BuildMetadata::new(&format!["{}.{}", self.branch, self.hash]).unwrap(),
            ..parse_blender_ver(&self.version, false).unwrap()
        }
    }
}
