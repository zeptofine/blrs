use std::{fmt::Debug, fmt::Display, str::FromStr, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::{Regex, RegexBuilder};
use thiserror::Error;

/// WildPlacement is used to define a strategy on how to match elements in an unordered collection.
/// This has no `find` implementation like [OrdPlacement] does because it is
/// fairly straightforward for callers to implement.
#[derive(Debug, Clone, Default)]

pub enum WildPlacement<T: PartialEq> {
    /// This is analogous to doing nothing.
    #[default]
    Any,
    /// Find a specific value in a group.
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

/// OrdPlacement is used to define a strategy on how to match elements in an ordered collection. It can be:
///
/// ```
/// use blrs::search::OrdPlacement;
///
/// let v = vec![&0, &1, &4, &10, &65];
/// assert_eq![OrdPlacement::Latest.find(&v, |x| *v[x]), vec![65]];
/// assert_eq![OrdPlacement::Oldest.find(&v, |x| *v[x]), vec![0]];
/// assert_eq![OrdPlacement::Any.find(&v, |x| v[x]), v];
/// assert_eq![OrdPlacement::Exact(10).find(&v, |x| *v[x]), vec![10]];
///
/// ```
#[derive(Clone, Default)]
pub enum OrdPlacement<T: PartialOrd + PartialEq> {
    /// Find the latest/newest value in a group.
    Latest,
    /// This is analogous to doing nothing.
    #[default]
    Any,
    /// Find the oldest value in a group.
    Oldest,
    /// Find a specific value in a group.
    Exact(T),
}

impl<T: Ord + PartialOrd + PartialEq + Debug> OrdPlacement<T> {
    /// Filters the values and returns a [`Vec<R>`] that pass the placement check.
    ///
    /// The F function must take an index and return a value that the caller expects.
    pub fn find<F, R>(&self, values: &[&T], f: F) -> Vec<R>
    where
        F: Fn(usize) -> R,
        R: Debug,
    {
        match self {
            OrdPlacement::Latest => {
                let mut latest: Option<&T> = None;
                let mut all_latest = vec![];
                for (i, value) in values.iter().enumerate() {
                    if latest.is_some_and(|l| &l == value) {
                        all_latest.push(f(i));
                    } else if latest.is_some_and(|l| l < *value) | latest.is_none() {
                        all_latest = vec![f(i)];
                        latest = Some(value);
                    }
                    println!["{:?} {:?}", i, value];
                    println!["LATEST: {:?}", latest];
                    println!["ALL_LATEST: {:?}", all_latest];
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
///```md
/// *.*.*
///
/// 1.2.3-master
///
/// 4.^.^-stable@^
///
/// 4.3.^+cb886aba06d5@^
///
/// 4.3.^@2024-07-31T23:53:51+00:00
///
/// And of course, a full example:
///
/// 4.3.^-stable+cb886aba06d5@2024-07-31T23:53:51+00:00
///```
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
/// `(?:[\+\#]([\d\w]+))?`          -- build hash (optional)
///
/// `(?:\@([\dT\+\:Z\ \^\*\-]+))?`  -- commit time (saved as ^|*|- or an isoformat) (optional)
///  
/// `$`                             -- end of string

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

/// A Search query with the necessary parameters to group and filter
/// BasicBuildInfo instances.
#[derive(Debug, Clone, Default)]
pub struct VersionSearchQuery {
    /// The nickname of the repository that the build belongs to.
    pub repository: WildPlacement<String>,

    /// The major part of the build version.
    pub major: OrdPlacement<u64>,

    /// The minor part of the build version.
    ///
    /// In older versions, this can be double digits like 2.93
    pub minor: OrdPlacement<u64>,

    /// The patch part of the build version.
    ///
    /// This is usually omitted in older versions of blender because they
    /// used to follow a different naming scheme.
    pub patch: OrdPlacement<u64>,

    /// The branch of the build.
    /// Depending on the repo you use, this is less or more effective. It's mostly
    /// useful to differentiate build subgroups.
    pub branch: WildPlacement<String>,

    /// The build hash of the build.
    /// This tends to be a unique value per build, so it's a good value to
    /// restrict to ***one*** specific build.
    pub build_hash: WildPlacement<String>,

    /// A specific date in time to sort by.
    /// By personal testing, it is strongly advised to only use the ordered placement
    /// mode because of how specific the actual [`DateTime`] struct is.
    pub commit_dt: OrdPlacement<DateTime<Utc>>,
}

impl VersionSearchQuery {
    /// Returns a new [VersionSearchQuery] with a new [`OrdPlacement<DateTime<Utc>>`], defaulting to [OrdPlacement::Any].
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

#[derive(Debug, Clone, Error)]
/// Errors that could occur from giving an incorrect string to [`VersionSearchQuery::try_from`].
///
/// ```
/// use blrs::search::VersionSearchQuery;
/// use blrs::search::FromError;
/// assert![matches![VersionSearchQuery::try_from("*.*.*"), Ok(_)]];
/// assert![matches![VersionSearchQuery::try_from("incorrect!"), Err(FromError::CannotCaptureViaRegex)]];
/// ```
pub enum FromError {
    /// This can occur when the string could not be parsed by the [VERSION_SEARCH_REGEX].

    #[error("Could not get required parameters from the given string")]
    CannotCaptureViaRegex,
}

impl TryFrom<&str> for VersionSearchQuery {
    type Error = FromError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let captures = VERSION_SEARCH_REGEX
            .captures(value)
            .ok_or(Self::Error::CannotCaptureViaRegex)?;

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
            _ => return Err(FromError::CannotCaptureViaRegex),
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

        Ok(Self {
            major,
            minor,
            patch,
            repository,
            branch,
            build_hash,
            commit_dt,
        })
    }
}
