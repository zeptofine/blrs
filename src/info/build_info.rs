use std::{
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

use super::{get_info_from_blender, CollectedInfo};

static MATCHERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // <major>.<minor> (sub <patch>): 2.80 (sub 75) -> 2.80.75
        r"(?P<ma>\d+)\.(?P<mi>\d+) \(sub (?P<pa>\d+)\)",
        // <major>.<minor>.<patch> <Prerelease>   2.80.0 Alpha  -> 2.80.0-alpha
        r"(?P<ma>\d+)\.(?P<mi>\d+)\.(?P<pa>\d+)[ \-](?P<pre>[^+]*[^wli][^ndux][^s]?)",
        r"(?P<ma>\d+)\.(?P<mi>\d+)[ \-](?P<pre>[^+]*[^wli][^ndux][^s]?)",
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

pub fn simple_clean(s: &str) -> &str {
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
const OLDVER_CUTOFF: Version = Version {
    major: 2,
    minor: 83,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::EMPTY,
};

const FILE_VERSION: f32 = 1.0;

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
pub trait BlendBuild: Sized {
    fn branch(&self) -> &str;
    fn with_branch(self, branch: Option<&str>) -> Result<Self, semver::Error>;

    fn build_hash(&self) -> &str;
    fn with_build_hash(self, hash: Option<&str>) -> Result<Self, semver::Error>;

    fn display_version(&self) -> String;
    fn display_label(&self) -> String;
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct VerboseVersion {
    pub v: Version,
    hash_split: usize,
} // format: <major>.<minor>.<patch>[-pre][+build][.hash]

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
}

impl BlendBuild for VerboseVersion {
    fn with_branch(self, branch: Option<&str>) -> Result<Self, semver::Error> {
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

    fn with_build_hash(self, hash: Option<&str>) -> Result<Self, semver::Error> {
        let hash = hash.unwrap_or("ffffffff");

        Ok(Self {
            v: Version {
                build: BuildMetadata::new(&format!["{}.{}", self.branch(), hash])?,
                ..self.v
            },
            hash_split: self.hash_split,
        })
    }

    fn branch(&self) -> &str {
        &self.v.build[..self.hash_split]
    }

    fn build_hash(&self) -> &str {
        // &self.v.build
        &self.v.build[self.hash_split + 1..]
    }

    fn display_version(&self) -> String {
        let v = &self.v;
        if *v < OLDVER_CUTOFF {
            format!["{}.{}{}", v.major, v.minor, v.patch]
        } else {
            format!["{}.{}.{}", v.major, v.minor, v.patch]
        }
    }

    fn display_label(&self) -> String {
        match self.branch() {
            "lts" => "LTS".to_string(),
            "patch" | "experimental" | "daily" => {
                let prerelease = self.v.pre.to_string();
                if !prerelease.is_empty() {
                    prerelease
                } else {
                    self.branch().to_string()
                }
            }
            x => x.to_string(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Ord, Clone, Serialize, Deserialize)]
pub struct BasicBuildInfo {
    pub ver: VerboseVersion,
    pub commit_dt: DateTime<Utc>,
}

impl PartialOrd for BasicBuildInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.commit_dt.partial_cmp(&other.commit_dt) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.ver.partial_cmp(&other.ver)
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

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LocalBuildInfo {
    pub basic: BasicBuildInfo,
    pub is_favorited: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_exe: Option<String>,
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
            metadata: info.clone(),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct LocalBuild {
    pub folder: PathBuf,
    pub info: LocalBuildInfo,
}

impl LocalBuild {
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
    pub fn read_exact(filepath: &Path) -> Result<Self, io::Error> {
        let file = File::open(filepath)?;
        let bis: BuildInfoSpec = serde_json::from_reader(file)?;

        Ok(Self {
            folder: filepath.parent().unwrap().into(),
            info: bis.metadata,
        })
    }

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

    pub fn write(&self) -> Result<(), io::Error> {
        self.write_to(self.folder.join(".build_info"))
    }

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

    use crate::{info::parse_blender_ver, BlendBuild};

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
