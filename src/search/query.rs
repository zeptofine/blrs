use std::{fmt::Display, str::FromStr, sync::LazyLock};

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

#[derive(Debug, Clone, Default)]
pub enum OrdPlacement<T: PartialOrd + PartialEq> {
    Latest,
    #[default]
    Any,
    Oldest,
    Exact(T),
}

impl<T: Ord + PartialOrd + PartialEq> OrdPlacement<T> {
    pub fn search(&self, values: &[&T]) -> Vec<usize> {
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
/// `(?:\+([\d\w]+))?`              -- build hash (optional)
///
/// `(?:\@([\dT\+\:Z\ \^\*\-]+))?`  -- commit time (saved as ^|*|- or an isoformat) (optional)
///  
/// $                               -- end of string

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
    .ignore_whitespace(true)
    .build()
    .unwrap()
});

#[derive(Debug, Clone, Default)]
pub struct VersionSearchQuery {
    pub major: OrdPlacement<u64>,
    pub minor: OrdPlacement<u64>,
    pub patch: OrdPlacement<u64>,
    pub branch: WildPlacement<String>,
    pub build_hash: WildPlacement<String>,
    pub commit_dt: OrdPlacement<DateTime<Utc>>,
}

pub enum FromError {
    CannotCaptureViaRegex,
}

impl TryFrom<String> for VersionSearchQuery {
    type Error = FromError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let captures = VERSION_SEARCH_REGEX.captures(&value);
        if captures.is_none() {
            return Err(FromError::CannotCaptureViaRegex);
        }

        let captures = captures.unwrap();
        let (major, minor, patch) = match (captures.get(1), captures.get(2), captures.get(3)) {
            (Some(ma), Some(mi), Some(pa)) => (
                OrdPlacement::from(ma.as_str()),
                OrdPlacement::from(mi.as_str()),
                OrdPlacement::from(pa.as_str()),
            ),
            _ => return Err(FromError::CannotCaptureViaRegex),
        };

        let branch = captures
            .get(4)
            .map(|m| WildPlacement::from(m.as_str()))
            .unwrap_or_default();
        let build_hash = captures
            .get(5)
            .map(|m| WildPlacement::from(m.as_str()))
            .unwrap_or_default();

        let commit_dt = captures
            .get(6)
            .map(|m| OrdPlacement::from(m.as_str()))
            .unwrap_or_default();

        Ok(Self {
            major,
            minor,
            patch,
            branch,
            build_hash,
            commit_dt,
        })
    }
}
