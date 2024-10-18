use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use log::{debug, error};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    fetching::{build_repository::BuildRepo, build_schemas::builder_schema::BlenderBuildSchema},
    BLRSPaths, BasicBuildInfo, LocalBuild, RemoteBuild,
};

#[inline]
pub(crate) fn is_dir_or_link_to_dir(p: &Path) -> bool {
    p.is_dir() || p.read_link().is_ok_and(|p| p.is_dir() || !p.exists())
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildVariant<B: Display + Debug> {
    pub b: B,
    pub target_os: String,
    pub architecture: String,
    pub extension: String,
}

impl<B: Display + Debug> Display for BuildVariant<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "{}", self.b]
    }
}

#[derive(Clone, Serialize)]
pub struct Variants<B: Display + Debug> {
    pub v: Vec<BuildVariant<B>>,
    pub basic: BasicBuildInfo,
}

impl<B: Display + Debug> Debug for Variants<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variants")
            .field(
                "v",
                &self.v.iter().map(|v| format!["{}", v]).collect::<Vec<_>>(),
            )
            .field("basic", &self.basic)
            .finish()
    }
}

impl<B: Display + Debug> Variants<B> {
    pub fn filter_target(self, target: (&str, &str, &str)) -> Self {
        Self {
            v: self
                .v
                .into_iter()
                .filter(|build| {
                    build.target_os == target.0
                        && build.architecture == target.1
                        && build.extension == target.2
                })
                .collect(),
            basic: self.basic,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum BuildEntry {
    NotInstalled(Variants<RemoteBuild>),
    Installed(String, LocalBuild),
    Errored(#[serde(skip)] std::io::Error, Option<PathBuf>),
}

#[derive(Debug, Serialize)]
pub enum RepoEntry {
    Registered(BuildRepo, Vec<BuildEntry>),
    Unknown(String, Vec<BuildEntry>),
    Error(String, #[serde(skip)] std::io::Error),
}
impl RepoEntry {
    pub fn has_installed_builds(&self) -> bool {
        match self {
            RepoEntry::Registered(_, vec) | RepoEntry::Unknown(_, vec) => vec
                .iter()
                .any(|entry| matches![entry, BuildEntry::Installed(_, _)]),
            RepoEntry::Error(_, _) => false,
        }
    }
}

pub fn read_repo_cache(repo_cache_path: &Path) -> Vec<RemoteBuild> {
    match repo_cache_path.exists() {
        true => match File::open(repo_cache_path) {
            Ok(file) => {
                serde_json::from_reader::<_, Vec<BlenderBuildSchema>>(file).unwrap_or_default()
            }
            Err(_) => vec![],
        },
        false => vec![],
    }
    .into_iter()
    .map(RemoteBuild::from)
    .collect()
}

pub fn read_repo_cache_variants(repo_cache_path: &Path) -> HashMap<String, Variants<RemoteBuild>> {
    read_repo_cache(repo_cache_path)
        .into_iter()
        .sorted_by_key(|k| k.basic.ver.clone())
        .chunk_by(|k| k.basic.ver.clone())
        .into_iter()
        .map(|(v, g)| {
            (v.to_string(), {
                let variants: Vec<BuildVariant<RemoteBuild>> = g
                    .filter(|b| !b.file_extension.as_ref().is_some_and(|e| e == "sha256"))
                    .map(|rb| BuildVariant {
                        target_os: rb.platform.clone().unwrap_or_default(),
                        architecture: rb.architecture.clone().unwrap_or_default(),
                        extension: rb.file_extension.clone().unwrap_or_default(),
                        b: rb,
                    })
                    .collect();
                if !variants.is_empty() {
                    let first = &variants[0];
                    let basic = first.b.basic.clone();
                    Some(Variants { v: variants, basic })
                } else {
                    None
                }
            })
        })
        .filter_map(|(s, variants)| variants.map(|v| (s, v)))
        .collect()
}

fn read_local_entries(repo_library_path: &Path) -> Result<Vec<BuildEntry>, std::io::Error> {
    Ok(repo_library_path
        .read_dir()?
        .filter_map(|item| match item {
            Ok(f) => match is_dir_or_link_to_dir(&f.path()) {
                true => Some(
                    match LocalBuild::read(&f.path().read_link().unwrap_or(f.path())) {
                        Ok(build) => BuildEntry::Installed(
                            f.file_name().to_str().unwrap().to_string(),
                            build,
                        ),
                        Err(e) => BuildEntry::Errored(e, Some(f.path())),
                    },
                ),
                false => None,
            },

            Err(e) => Some(BuildEntry::Errored(e, None)),
        })
        .collect())
}

fn get_known_and_unknown_repos(
    repos: Vec<BuildRepo>,
    paths: &BLRSPaths,
) -> std::io::Result<Vec<Result<BuildRepo, String>>> {
    let mut repo_map: HashMap<String, BuildRepo> =
        repos.into_iter().map(|r| (r.repo_id.clone(), r)).collect();

    let folders: HashSet<String> = paths
        .library
        .read_dir()
        .inspect_err(|e| error!("Failed to read {:?}: {}", paths.library, e))?
        .filter_map(|item| {
            let item = item.ok()?;
            is_dir_or_link_to_dir(&item.path())
                .then(|| item.file_name().to_str().unwrap().to_string())
        })
        .collect();

    let existing: Vec<Result<_, _>> = folders
        .into_iter()
        .map(|s| match repo_map.remove(&s) {
            Some(r) => Ok(r),
            None => Err(s),
        })
        .collect();

    let missing: Vec<Result<_, _>> = repo_map.into_values().map(Ok).collect();

    Ok(existing.into_iter().chain(missing).collect())
}

pub fn read_repos(
    repos: Vec<BuildRepo>,
    paths: &BLRSPaths,
    installed_only: bool,
) -> std::io::Result<Vec<RepoEntry>> {
    let registered = get_known_and_unknown_repos(repos, paths)?;

    Ok(registered
        .into_iter()
        .map(|r| {
            debug!("Evaluating {:?}", r);
            let id = match &r {
                Ok(r) => r.repo_id.clone(),
                Err(s) => s.clone(),
            };

            let library_path = paths.library.join(&id);
            let entries = read_local_entries(&library_path);
            let cache_path = paths.remote_repos.join(id.clone() + ".json");
            let remote_variants = read_repo_cache_variants(&cache_path)
                .into_iter()
                .map(|(s, v)| (s, BuildEntry::NotInstalled(v)));

            match (r, entries) {
                (Ok(r), Ok(mut entries)) => {
                    if !installed_only {
                        entries = entries
                            .into_iter()
                            .map(|e| match &e {
                                BuildEntry::Installed(_dir, local_build) => {
                                    (local_build.info.basic.ver.to_string(), e)
                                }
                                BuildEntry::Errored(_, _) => (Uuid::new_v4().to_string(), e),
                                BuildEntry::NotInstalled(_) => unreachable!(),
                            })
                            .chain(remote_variants)
                            .unique_by(|(s, _)| s.clone())
                            .map(|(_, e)| e)
                            .collect();
                    }
                    RepoEntry::Registered(r.clone().clone(), entries)
                }
                (Ok(r), Err(_)) => {
                    RepoEntry::Registered(r, remote_variants.map(|(_, v)| v).collect())
                }
                (Err(name), Ok(entries)) => RepoEntry::Unknown(name, entries),
                (Err(name), Err(err)) => RepoEntry::Error(name, err),
            }
        })
        .collect())
}
