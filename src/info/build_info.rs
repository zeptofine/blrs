use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Display,
    fs::File,
    hash::Hash,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};

use crate::search::{OrdPlacement, VersionSearchQuery, WildPlacement};

use super::{get_info_from_blender, CollectedInfo, VerboseVersion};

static MATCHERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // <major>.<minor> (sub <patch>): 2.80 (sub 75) -> 2.80.75
        r"(?P<ma>\d+)\.(?P<mi>\d+) \(sub (?P<pa>\d+)\)",
        // <major>.<minor>.<patch> <Prerelease>   2.80.0 Alpha  -> 2.80.0-alpha
        r"(?P<ma>\d+)\.(?P<mi>\d+)\.(?P<pa>\d+)[ \-](?P<pre>[^+]*)",
        r"(?P<ma>\d+)\.(?P<mi>\d+)[ \-](?P<pre>[^+]*)",
        // <major>.<minor>: 2.79 -> 2.79.0
        r"(?P<ma>\d+)\.(?P<mi>\d+)$",
        // <major>.<minor><[chars]*(1-3)>: 2.79rc1 -> 2.79.0-rc1
        r"(?P<ma>\d+)\.(?P<mi>\d+)(?P<pre>[^-]{0,3})",
        // <major>.<minor><patch?> 2.79 -> 2.79.0 | 2.79b -> 2.79.0-b
        r"(?P<ma>\d+)\.(?P<mi>\d+)(?P<pre>\D[^\.\s]*)?",
    ]
    .into_iter()
    .map(|re| Regex::new(re).unwrap())
    .collect()
});

static INITIAL_CLEANER1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:blender-?|Blender|BLENDER|v)-?(\d.*)").unwrap());

/// Cleans a version string by removing extraneous information like platform identifiers and "v" prefixes.
///
/// This function aims to standardize Blender version strings for easier comparison and handling.
fn simple_clean(s: &str) -> &str {
    let mut s = s;

    let c = INITIAL_CLEANER1.captures(s);
    if let Some(c) = c {
        s = c.get(1).unwrap().as_str();
    }

    if let Some(i) = s.find("-windows") {
        s = &s[..i];
    }
    if let Some(i) = s.find("-linux") {
        s = &s[..i];
    }

    s
}

/// This describes the first version that adopted the new SemVer compatible
/// versioning scheme. Before that, it was seemingly arbitrary
/// with a major version, a minor version, and sometimes an a or a b slapped to the end.
pub const OLDVER_CUTOFF: Version = Version {
    major: 2,
    minor: 83,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::EMPTY,
};

const FILE_VERSION: f32 = 1.0;

/// Parses a Blender version string into a `semver::Version` object.
///
/// This function handles various formats of Blender version strings, including older, non-SemVer compatible versions.
/// It uses regular expressions to extract the major, minor, patch, and prerelease information from the input string.
/// If the string cannot be parsed into a valid `Version` object, it returns `None`.
pub fn parse_blender_ver(s: &str, search: bool) -> Option<Version> {
    let mut s = s.trim();
    if let Ok(v) = Version::parse(s) {
        return Some(v);
    }

    s = simple_clean(s);

    if let Ok(v) = Version::parse(s) {
        return Some(v);
    }

    let g = if search {
        MATCHERS.iter().find_map(|re| re.captures(s))
    } else {
        MATCHERS.iter().find_map(|re| re.captures_at(s, 0))
    };

    match g {
        Some(g) => {
            let major = g.name("ma")?.as_str().parse::<u64>().ok()?;
            let minor = g.name("mi")?.as_str().parse::<u64>().ok()?;
            let patch = g
                .name("pa")
                .map(|m| m.as_str())
                .unwrap_or("0")
                .parse::<u64>()
                .ok()?;
            let mut v = Version::new(major, minor, patch);
            v.pre = match g.name("pre") {
                None => Prerelease::EMPTY,
                Some(m) => Prerelease::from_str(&m.as_str().to_lowercase()).unwrap(),
            };

            Some(v)
        }

        None => None,
    }
}

/// The most important information of a Blender build. Paramount to most of the project.
#[derive(Hash, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct BasicBuildInfo {
    /// The version packed with extra information, such as the branch and build hash.
    pub ver: VerboseVersion,
    /// The date and time when the commit was made.
    pub commit_dt: DateTime<Utc>,
}

impl BasicBuildInfo {
    /// Get the underlying Version struct from the [`VerboseVersion`].
    pub fn version(&self) -> &Version {
        self.ver.v()
    }
}
impl AsRef<Self> for BasicBuildInfo {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl PartialOrd for BasicBuildInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BasicBuildInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.commit_dt.cmp(&other.commit_dt) {
            Ordering::Equal => self.ver.cmp(&other.ver),
            ord => ord,
        }
    }
}

impl Default for BasicBuildInfo {
    fn default() -> Self {
        BasicBuildInfo {
            ver: VerboseVersion::default(),
            commit_dt: Utc::now(),
        }
    }
}

impl Display for BasicBuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "{}", self.ver]
    }
}

impl From<BasicBuildInfo> for VersionSearchQuery {
    fn from(val: BasicBuildInfo) -> Self {
        VersionSearchQuery {
            repository: WildPlacement::Any,
            major: OrdPlacement::Exact(val.version().major),
            minor: OrdPlacement::Exact(val.version().minor),
            patch: OrdPlacement::Exact(val.version().patch),
            branch: WildPlacement::Exact(val.ver.branch().to_string()),
            build_hash: WildPlacement::Exact(val.ver.build_hash().to_string()),
            commit_dt: OrdPlacement::Exact(val.commit_dt),
        }
    }
}

/// Info of a local build, including extra values relating to personal preference.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LocalBuildInfo {
    ///  The basic build information for this local build.
    pub basic: BasicBuildInfo,

    /// Whether or not this build is a favorite.
    pub is_favorited: bool,

    /// An optional custom name for the build.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,

    /// An optional custom executable path for this build.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_exe: Option<String>,

    /// An optional set of custom environment variables to use when running this build.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_env: Option<HashMap<String, String>>,
}

/// This is what a normal `.build_info` file looks like.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildInfoSpec {
    file_version: f32,
    metadata: LocalBuildInfo,
}

impl From<LocalBuildInfo> for BuildInfoSpec {
    fn from(info: LocalBuildInfo) -> Self {
        BuildInfoSpec {
            file_version: FILE_VERSION,
            metadata: info,
        }
    }
}
#[derive(PartialEq, Debug, Clone, Serialize)]
/// A combination of the folder and local build info.
pub struct LocalBuild {
    /// The path to the build's directory.
    pub folder: PathBuf,
    /// Metadata about this build.
    pub info: LocalBuildInfo,
}

impl AsRef<BasicBuildInfo> for LocalBuild {
    fn as_ref(&self) -> &BasicBuildInfo {
        &self.info.basic
    }
}

impl LocalBuild {
    /// Reads a `LocalBuild` instance from either a `.build_info` file in the current directory or
    /// within a given folder.
    pub fn read(file_or_folder: &Path) -> Result<Self, io::Error> {
        if file_or_folder
            .file_name()
            .is_some_and(|name| name == ".build_info")
        {
            Self::read_exact(file_or_folder)
        } else {
            Self::read_exact(&file_or_folder.join(".build_info"))
        }
    }

    /// Reads a `LocalBuild` instance from the specified `.build_info` file path.
    pub fn read_exact(filepath: &Path) -> Result<Self, io::Error> {
        let file = File::open(filepath)?;
        let bis: BuildInfoSpec = serde_json::from_reader(file)?;

        Ok(Self {
            folder: filepath.parent().unwrap().into(),
            info: bis.metadata,
        })
    }

    /// Attempts to generate a `LocalBuild` instance from an executable's path by extracting information
    /// about the build using Blender's internal metadata.
    pub fn generate_from_exe(executable: &Path) -> io::Result<LocalBuild> {
        let build_path = executable.parent().unwrap();

        get_info_from_blender(executable).and_then(|info| match info {
            CollectedInfo {
                commit_dt: Some(commit_dt),
                build_hash,
                branch,
                subversion: Some(v),
                custom_name,
            } => {
                let v = VerboseVersion::new(
                    v.major,
                    v.minor,
                    v.patch,
                    match &branch {
                        Some(s) => Some(s.as_str()),
                        None => None,
                    },
                    None,
                    match &build_hash {
                        Some(s) => Some(s.as_str()),
                        None => None,
                    },
                );

                let mut basic_info = BasicBuildInfo { ver: v, commit_dt };
                if let Some(hash) = build_hash {
                    basic_info.ver = basic_info.ver.with_build_hash(Some(&hash)).unwrap()
                };
                if let Some(branch) = branch {
                    basic_info.ver = basic_info.ver.with_branch(Some(&branch)).unwrap()
                }

                let local_info = LocalBuildInfo {
                    basic: basic_info,
                    is_favorited: false,
                    custom_name,
                    custom_exe: None,
                    custom_env: None,
                };

                let local_build = LocalBuild {
                    folder: build_path.to_path_buf(),
                    info: local_info,
                };

                Ok(local_build)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Could not get all necessary info from blender",
            )),
        })
    }

    /// Writes the current `LocalBuild` instance to a `.build_info` file.
    pub fn write(&self) -> Result<(), io::Error> {
        self.write_to(self.folder.join(".build_info"))
    }

    /// Writes the current `LocalBuild` instance to a given file path.
    pub fn write_to(&self, filepath: PathBuf) -> Result<(), io::Error> {
        let data = serde_json::to_string(&BuildInfoSpec::from(self.info.clone())).unwrap();

        let mut file = File::create(filepath)?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use semver::{BuildMetadata, Prerelease, Version};

    use crate::info::parse_blender_ver;

    use super::VerboseVersion;

    const TEST_STRINGS: LazyLock<[(&str, Version); 12]> = LazyLock::new(|| {
        [
            ("Blender1.0", Version::parse("1.0.0").unwrap()),
            (
                "blender-4.3.0-alpha-linux",
                Version::parse("4.3.0-alpha").unwrap(),
            ),
            ("3.6.14", Version::parse("3.6.14").unwrap()),
            (
                "4.3.0-alpha+daily.ddc9f92777cd",
                Version {
                    major: 4,
                    minor: 3,
                    patch: 0,
                    pre: Prerelease::new("alpha").unwrap(),
                    build: BuildMetadata::new("daily.ddc9f92777cd").unwrap(),
                },
            ),
            (
                "blender-3.3.21-stable+v33.e016c21db151-linux.x86_64-release.tar.xz",
                Version {
                    major: 3,
                    minor: 3,
                    patch: 21,
                    pre: Prerelease::new("stable").unwrap(),
                    build: BuildMetadata::new("v33.e016c21db151").unwrap(),
                },
            ),
            (
                "blender-4.3.0-alpha+daily.d9c941a464e7",
                Version {
                    major: 4,
                    minor: 3,
                    patch: 0,
                    pre: Prerelease::new("alpha").unwrap(),
                    build: BuildMetadata::new("daily.d9c941a464e7").unwrap(),
                },
            ),
            (
                "blender-4.1.0-linux-x64.tar.xz",
                Version {
                    major: 4,
                    minor: 1,
                    patch: 0,
                    pre: Prerelease::new("").unwrap(),
                    build: BuildMetadata::new("").unwrap(),
                },
            ),
            ("2.80 (sub 75)", Version::parse("2.80.75").unwrap()),
            ("2.79", Version::parse("2.79.0").unwrap()),
            ("2.79rc1", Version::parse("2.79.0-rc1").unwrap()),
            ("2.79b", Version::parse("2.79.0-b").unwrap()),
            ("v4.2.2", Version::parse("4.2.2").unwrap()),
        ]
    });

    #[test]
    fn test_parser() {
        println!["{:#?}", TEST_STRINGS];
        TEST_STRINGS.iter().for_each(|(s, v)| {
            let estimated_version = parse_blender_ver(s, true);
            println!["{:?} -> {:?}", s, estimated_version];
            assert_eq!(estimated_version.unwrap(), *v);
        })
    }

    #[test]
    fn test_blend_build_methods() {
        let ver = VerboseVersion::default();

        println!["{:?}", ver];
        assert_eq!(ver.branch(), "null");
        assert_eq!(ver.build_hash(), "ffffffff");
    }
}
