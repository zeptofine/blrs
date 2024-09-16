use std::collections::HashMap;

use chrono::DateTime;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

use crate::{
    info::{build_info::VerboseVersion, parse_blender_ver},
    BasicBuildInfo, RemoteBuild,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct RemoteBuildSchema {
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

impl From<RemoteBuildSchema> for RemoteBuild {
    fn from(val: RemoteBuildSchema) -> Self {
        RemoteBuild {
            link: val.url.clone(),
            info: BasicBuildInfo {
                ver: VerboseVersion::from(val.full_version()),
                commit_dt: DateTime::from_timestamp(val.file_mtime as i64, 0).unwrap(),
            },
        }
    }
}

impl RemoteBuildSchema {
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
#[derive(Debug, Default)]
pub struct Sha256Pair {
    pub sha256: Option<RemoteBuildSchema>,
    pub build: Option<RemoteBuildSchema>,
}

pub fn get_sha256_pairs(lst: Vec<RemoteBuildSchema>) -> HashMap<Version, Sha256Pair> {
    let mut map: HashMap<Version, Sha256Pair> = HashMap::new();

    for schema in lst {
        let ver = schema.full_version_and_platform();

        let entry = map.remove(&ver);
        if schema.file_extension == "sha256" {
            map.insert(
                ver,
                Sha256Pair {
                    sha256: Some(schema),
                    build: match entry {
                        Some(e) => e.build,
                        None => None,
                    },
                },
            );
        } else {
            map.insert(
                ver,
                Sha256Pair {
                    sha256: match entry {
                        Some(e) => e.sha256,
                        None => None,
                    },
                    build: Some(schema),
                },
            );
        }
    }

    map
}
