use std::{fmt::Debug, fmt::Display, str::FromStr, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::{Regex, RegexBuilder};

#[derive(Debug, Clone, Default)]
pub enum WildPlacement<T: PartialEq> {
    #[default]
    Any,
    Exact(T),
}

impl<T: Display + PartialEq> Display for WildPlacement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            WildPlacement::Any => "*".to_string(),
            WildPlacement::Exact(t) => format!["{t}"],
        })
    }
}

impl<T: FromStr + PartialEq> From<&str> for WildPlacement<T> {
    fn from(value: &str) -> Self {
        match value.trim() {
            "*" => WildPlacement::Any,
            s => match s.parse::<T>() {
                Ok(t) => WildPlacement::Exact(t),
                Err(_) => WildPlacement::Any,
            },
        }
    }
}

#[derive(Clone, Default)]
pub enum OrdPlacement<T: PartialOrd + PartialEq> {
    Latest,
    #[default]
    Any,
    Oldest,
    Exact(T),
}

impl<T: Ord + PartialOrd + PartialEq> OrdPlacement<T> {
    pub fn find<'a, F, R>(&self, values: &[&'a T], f: F) -> Vec<R>
    where
        F: Fn(usize) -> R,
    {
        match self {
            OrdPlacement::Latest => {
                let mut latest: Option<&T> = None;
                let mut all_latest = vec![];
                for (i, value) in values.iter().enumerate() {
                    if latest.is_some_and(|l| &l == value) {
                        all_latest.push(f(i));
                    } else if latest.is_some_and(|l| &l < value) | latest.is_none() {
                        all_latest = vec![f(i)];
                        latest = Some(value);
                    }
                }
                all_latest
            }
            OrdPlacement::Any => (0..values.len()).map(f).collect(),
            OrdPlacement::Oldest => {
                let mut oldest: Option<&T> = None;
                let mut all_oldest = vec![];
                for (i, value) in values.iter().enumerate() {
                    if oldest.is_some_and(|l| &l == value) {
                        all_oldest.push(f(i));
                    } else if oldest.is_some_and(|l| &l > value) | oldest.is_none() {
                        all_oldest = vec![f(i)];
                        oldest = Some(value);
                    }
                }
                all_oldest
            }
            OrdPlacement::Exact(t) => (0..values.len())
                .filter_map(|i| (values[i] == t).then_some(f(i)))
                .collect(),
        }
    }
}

impl<T: Display + PartialOrd + PartialEq> Display for OrdPlacement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            OrdPlacement::Latest => "^".to_string(),
            OrdPlacement::Any => "*".to_string(),
            OrdPlacement::Oldest => "-".to_string(),
            OrdPlacement::Exact(x) => x.to_string(),
        })
    }
}

impl<T: Debug + PartialOrd + PartialEq> Debug for OrdPlacement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            OrdPlacement::Latest => "Latest (^)".to_string(),
            OrdPlacement::Any => "Any (*)".to_string(),
            OrdPlacement::Oldest => "Oldest (-)".to_string(),
            OrdPlacement::Exact(x) => format!["Exact ({x:?})"],
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

/// Regex breakdown:
///
/// `^`                             -- start of string
///
/// `([\^\-\*]|\d+)1`            x3 -- major, minor, and patch (required)
///
/// `(?:\-([^\@\s\+]+))?`           -- branch (optional)
///
/// `(?:[\+\#]([\d\w]+))?`              -- build hash (optional)
///
/// `(?:\@([\dT\+\:Z\ \^\*\-]+))?`  -- commit time (saved as ^|*|- or an isoformat) (optional)
///  
/// $                               -- end of string

pub static VERSION_SEARCH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"^
        (?:([^/]+)/)?
    ([\^\-\*]|\d+)\.([\^\-\*]|\d+)\.([\^\-\*]|\d+)
    (?:\-([^@\s\+]+))?
    (?:[\+\#]([\d\w\^\-\*]+))?
    (?:@([\^\-\*]|[\d\+:ZUTC \-\^]+))?
    $",
    )
    .case_insensitive(true)
    .ignore_whitespace(true)
    .build()
    .unwrap()
});

#[derive(Debug, Clone, Default)]
pub struct VersionSearchQuery {
    pub repository: WildPlacement<String>,
    pub major: OrdPlacement<u64>,
    pub minor: OrdPlacement<u64>,
    pub patch: OrdPlacement<u64>,
    pub branch: WildPlacement<String>,
    pub build_hash: WildPlacement<String>,
    pub commit_dt: OrdPlacement<DateTime<Utc>>,
}

impl VersionSearchQuery {
    pub fn with_commit_dt(self, commit_dt: Option<OrdPlacement<DateTime<Utc>>>) -> Self {
        Self {
            commit_dt: commit_dt.unwrap_or_default(),
            ..self
        }
    }
}

impl Display for VersionSearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format![
            "{}.{}.{}-{}#{}",
            self.major, self.minor, self.patch, self.branch, self.build_hash,
        ];
        match &self.commit_dt {
            OrdPlacement::Latest | OrdPlacement::Oldest => s = format!["{}@{}", s, &self.commit_dt],
            OrdPlacement::Any => {}
            OrdPlacement::Exact(_) => {}
        }
        if let WildPlacement::Exact(repo) = &self.repository {
            s = format!["{}/{}", repo, s];
        }

        f.write_str(&s)
    }
}

#[derive(Debug, Clone)]
pub enum FromError {
    CannotCaptureViaRegex,
    CannotCaptureVersionNumbers,
}

impl TryFrom<&str> for VersionSearchQuery {
    type Error = FromError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let captures = VERSION_SEARCH_REGEX
            .captures(&value)
            .ok_or_else(|| Self::Error::CannotCaptureViaRegex)?;

        let repository = captures
            .get(1)
            .map(|m| WildPlacement::from(m.as_str()))
            .unwrap_or_default();

        let (major, minor, patch) = match (captures.get(2), captures.get(3), captures.get(4)) {
            (Some(ma), Some(mi), Some(pa)) => (
                OrdPlacement::from(ma.as_str()),
                OrdPlacement::from(mi.as_str()),
                OrdPlacement::from(pa.as_str()),
            ),
            _ => return Err(FromError::CannotCaptureVersionNumbers),
        };

        let branch = captures
            .get(5)
            .map(|m| WildPlacement::from(m.as_str()))
            .unwrap_or_default();
        let build_hash = captures
            .get(6)
            .map(|m| WildPlacement::from(m.as_str()))
            .unwrap_or_default();

        let commit_dt = captures
            .get(7)
            .map(|m| OrdPlacement::from(m.as_str()))
            .unwrap_or_default();

        let version_search_query = Ok(Self {
            major,
            minor,
            patch,
            repository,
            branch,
            build_hash,
            commit_dt,
        });
        version_search_query
    }
}
