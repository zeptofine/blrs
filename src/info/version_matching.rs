use std::{
    fmt::{Debug, Display},
    str::FromStr,
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use regex::{Regex, RegexBuilder};

use crate::info::{BasicBuildInfo, BlendBuild};

/// VersionSearchQuery syntax (NOT SEMVER COMPATIBLE!)
///
/// - `^`    | Match the largest/newest item in that column
/// - `*`    | Match any item in that column
/// - `-`    | Match the smallest/oldest item in that column
/// - `<n>`  | Match a specific item in that column
///
/// Valid examples of version search queries are:
///
/// `*.*.*`
///
/// `1.2.3-master`
///
/// `4.^.^-stable@^`
///
/// `4.3.^+cb886aba06d5@^`
///
/// `4.3.^@2024-07-31T23:53:51+00:00`
///
/// And of course, a full example:
///
/// `4.3.^-stable+cb886aba06d5@2024-07-31T23:53:51+00:00`
///
pub const VERSION_SEARCH_SYNTAX: &str =
    "<major_num>.<minor>.<patch>[-<branch>][+<build_hash>][@<commit time>]";

// # Regex breakdown:
// # ^                                     -- start of string
// # ([\^\-\*]|\d+)                     x3 -- major, minor, and patch (required)
// # (?:\-([^\@\s\+]+))?                   -- branch (optional)
// # (?:\+([\d\w]+))?                      -- build hash (optional)
// # (?:\@([\dT\+\:Z\ \^\*\-]+))?            -- commit time (saved as ^|*|- or an isoformat) (optional)
// # $                                     -- end of string

pub static VERSION_SEARCH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"^
    ([\^\-\*]|\d+)\.([\^\-\*]|\d+)\.([\^\-\*]|\d+)
    (?:\-([^\@\s\+]+))?
    (?:\+([\d\w]+))?
    (?:\@([\^\-\*]|[\dT\+\:Z\ \^\-]+))?
    $",
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

#[derive(Debug)]
pub enum WildPlacement<T: PartialEq> {
    Any,
    Exact(T),
}

#[derive(Debug)]
pub enum OrdPlacement<T: PartialOrd + PartialEq> {
    Latest,
    Any,
    Oldest,
    Exact(T),
}

impl<T: Ord + PartialOrd + PartialEq> OrdPlacement<T> {
    fn search(&self, values: &[&T]) -> Vec<usize> {
        match self {
            OrdPlacement::Latest => {
                let mut latest_index: Option<usize> = None;
                for (i, value) in values.iter().enumerate() {
                    if latest_index.is_none() || value > &values[latest_index.unwrap()] {
                        latest_index = Some(i);
                    }
                }
                latest_index.map(|i| vec![i]).unwrap_or_default()
            }
            OrdPlacement::Any => (0..values.len()).collect(),
            OrdPlacement::Oldest => todo!(),
            OrdPlacement::Exact(_) => todo!(),
        }
    }
}

impl<T: Display + PartialOrd + PartialEq> Display for OrdPlacement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            OrdPlacement::Latest => "Latest (^)".to_string(),
            OrdPlacement::Any => "Any (*)".to_string(),
            OrdPlacement::Oldest => "Oldest (-)".to_string(),
            OrdPlacement::Exact(x) => format!["Exact ({x})"],
        })
    }
}

impl<T: FromStr + PartialOrd + PartialEq> From<&str> for OrdPlacement<T> {
    fn from(s: &str) -> Self {
        match s {
            "Latest" | "^" => OrdPlacement::Latest,
            "Any" | "*" => OrdPlacement::Any,
            "Oldest" | "-" => OrdPlacement::Oldest,
            x => match x.parse::<T>() {
                Ok(t) => OrdPlacement::Exact(t),
                Err(_) => OrdPlacement::Any,
            },
        }
    }
}

pub struct VersionSearchQuery {
    pub major: OrdPlacement<u64>,
    pub minor: OrdPlacement<u64>,
    pub patch: OrdPlacement<u64>,
    pub branch: Option<WildPlacement<String>>,
    pub commit_hash: Option<WildPlacement<String>>,
    pub commit_dt: Option<OrdPlacement<DateTime<Utc>>>,
}

pub struct BInfoMatcher<'a, 'b> {
    versions: &'a [&'b BasicBuildInfo],
}

impl<'a, 'b> BInfoMatcher<'a, 'b> {
    fn new(versions: &'a [&'b BasicBuildInfo]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn match_all(&self, query: &VersionSearchQuery) -> Vec<&BasicBuildInfo> {
        let mut versions = Vec::from(self.versions);

        // check build_hash
        versions = if let Some(WildPlacement::Exact(h)) = &query.commit_hash {
            let vs: Vec<_> = versions
                .into_iter()
                .filter(|v| match v.build_hash() {
                    Some(bh) => bh == h,
                    None => false,
                })
                .collect();

            vs
        } else {
            versions
        };

        // check branch
        versions = if let Some(WildPlacement::Exact(b)) = &query.branch {
            let vs: Vec<_> = versions
                .into_iter()
                .filter(|v| match v.branch() {
                    Some(br) => br == b,
                    None => false,
                })
                .collect();

            vs
        } else {
            versions
        };

        // check versions
        versions = match query.major {
            OrdPlacement::Any => versions,
            _ => query
                .major
                .search(
                    &(versions
                        .iter()
                        .map(|v| &v.version.major)
                        .collect::<Vec<_>>()),
                )
                .into_iter()
                .map(|i| versions[i])
                .collect(),
        };
        versions = match query.minor {
            OrdPlacement::Any => versions,
            _ => query
                .minor
                .search(
                    &(versions
                        .iter()
                        .map(|v| &v.version.minor)
                        .collect::<Vec<_>>()),
                )
                .into_iter()
                .map(|i| versions[i])
                .collect(),
        };
        versions = match query.patch {
            OrdPlacement::Any => versions,
            _ => query
                .patch
                .search(
                    &(versions
                        .iter()
                        .map(|v| &v.version.patch)
                        .collect::<Vec<_>>()),
                )
                .into_iter()
                .map(|i| versions[i])
                .collect(),
        };

        println!["{:#?}", versions];

        // check commit time
        versions = query
            .commit_dt
            .as_ref()
            .map(|placement| match placement {
                OrdPlacement::Any => versions,
                p => p
                    .search(&(versions.iter().map(|v| &v.commit_dt).collect::<Vec<_>>()))
                    .into_iter()
                    .map(|i| versions[i])
                    .collect(),
            })
            .unwrap_or_default();

        versions
    }
}
