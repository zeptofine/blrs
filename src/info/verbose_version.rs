use semver::Prerelease;

use semver::BuildMetadata;
use serde::Deserialize;
use serde::Serialize;

use std::fmt::Display;

use semver::Version;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
/// A struct representing a version number with additional information about the build and branch.
pub struct VerboseVersion {
    v: Version,
    /// Index separating the build and hash in the build metadata string.
    hash_split: usize,
}

impl Default for VerboseVersion {
    fn default() -> Self {
        Self::new(0, 0, 0, None, None, None)
    }
}

impl Display for VerboseVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "{}", self.v]
    }
}

impl From<Version> for VerboseVersion {
    fn from(value: Version) -> Self {
        // Split the build metadata into the build and hash
        let (build, hash) = value.build.split_once('.').unwrap_or(("null", "ffffffff"));
        let hash_split = build.len();
        let metadata = BuildMetadata::new(&format!["{}.{}", build, hash]).unwrap_or_default();

        Self {
            v: Version {
                build: metadata,
                ..value
            },
            hash_split,
        }
    }
}

impl VerboseVersion {
    /// Creates a new VerboseVersion with the specified major, minor, patch, pre-release, build, and hash values.
    pub fn new(
        major: u64,
        minor: u64,
        patch: u64,
        pre: Option<&str>,
        build: Option<&str>,
        hash: Option<&str>,
    ) -> Self {
        let pre = pre
            .and_then(|p| Prerelease::new(p).ok())
            .unwrap_or_default();
        let build = build.unwrap_or("null");
        let hash = hash.unwrap_or("ffffffff");

        let hash_split = build.len();

        let build = BuildMetadata::new(&format!["{}.{}", build, hash]).unwrap_or_default();

        Self {
            v: Version {
                major,
                minor,
                patch,
                pre,
                build,
            },
            hash_split,
        }
    }

    /// Grab a reference of the underlying Version.
    pub fn v(&self) -> &Version {
        &self.v
    }

    /// Retrieves the branch string.
    pub fn branch(&self) -> &str {
        &self.v.build[..self.hash_split]
    }

    /// Retrieves the build hash string.
    pub fn build_hash(&self) -> &str {
        &self.v.build[self.hash_split + 1..]
    }

    /// Updates the VerboseVersion with a provided branch, returning an Ok result containing the updated version.
    /// Returns an error if the branch cannot be parsed as valid.
    pub fn with_branch(self, branch: Option<&str>) -> Result<Self, semver::Error> {
        let branch = branch.unwrap_or("null");
        let hash_split = branch.len();

        Ok(Self {
            v: Version {
                build: BuildMetadata::new(&format!["{}.{}", branch, self.build_hash()])?,
                ..self.v
            },
            hash_split,
        })
    }

    /// Updates the VerboseVersion with a provided build hash, returning an Ok result containing the updated version.
    /// Returns an error if the hash cannot be parsed as valid.
    pub fn with_build_hash(self, hash: Option<&str>) -> Result<Self, semver::Error> {
        let hash = hash.unwrap_or("ffffffff");

        Ok(Self {
            v: Version {
                build: BuildMetadata::new(&format!["{}.{}", self.branch(), hash])?,
                ..self.v
            },
            hash_split: self.hash_split,
        })
    }
}
