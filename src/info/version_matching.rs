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

#[derive(Debug, Clone)]
pub enum WildPlacement<T: PartialEq> {
    Any,
    Exact(T),
}

#[derive(Debug, Clone)]
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
    pub branch: WildPlacement<String>,
    pub commit_hash: WildPlacement<String>,
    pub commit_dt: OrdPlacement<DateTime<Utc>>,
}

pub struct BInfoMatcher<'a> {
    versions: &'a [BasicBuildInfo],
}

impl<'a> BInfoMatcher<'a> {
    fn new(versions: &'a [BasicBuildInfo]) -> Self {
        BInfoMatcher { versions }
    }

    pub fn match_all(&self, query: &VersionSearchQuery) -> Vec<&BasicBuildInfo> {
        let vs = self
            .versions
            .iter()
            .filter(|build| match query.commit_hash.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(hash) => build.build_hash().is_some_and(|bh| bh == hash),
            })
            .filter(|build| match query.branch.clone() {
                WildPlacement::Any => true,
                WildPlacement::Exact(branch) => build.branch().is_some_and(|br| br == branch),
            })
            .collect::<Vec<&BasicBuildInfo>>();

        let vs = match query.major {
            OrdPlacement::Any => vs,
            _ => query
                .major
                .search(&(vs.iter().map(|v| &v.version.major).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };
        let vs = match query.minor {
            OrdPlacement::Any => vs,
            _ => query
                .minor
                .search(&(vs.iter().map(|v| &v.version.minor).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };
        let vs = match query.patch {
            OrdPlacement::Any => vs,
            _ => query
                .patch
                .search(&(vs.iter().map(|v| &v.version.patch).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };

        let vs = match query.commit_dt {
            OrdPlacement::Any => vs,
            _ => query
                .commit_dt
                .search(&(vs.iter().map(|v| &v.commit_dt).collect::<Vec<_>>()))
                .into_iter()
                .map(|i| vs[i])
                .collect(),
        };

        vs
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use chrono::DateTime;
    use semver::Version;

    use crate::info::BasicBuildInfo;

    use super::BInfoMatcher;

    static builds: LazyLock<Vec<BasicBuildInfo>> = LazyLock::new(|| {
        vec![
            BasicBuildInfo {
                version: Version::parse("1.2.3+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-05-04T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.2+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-04-02T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.1+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-03-01T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("1.2.4+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2020-06-03T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("3.6.14+lts").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-16T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.2.0+stable").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-16T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.0+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-30T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.0+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-28T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
            BasicBuildInfo {
                version: Version::parse("4.3.1+daily").unwrap(),
                commit_dt: DateTime::parse_from_rfc3339("2024-07-20T00:00:00+00:00")
                    .unwrap()
                    .to_utc(),
            },
        ]
    });

    #[test]
    fn test_binfo_matcher() {
        let bs = builds.clone();
        let matcher = BInfoMatcher::new(&bs);
    }
}
