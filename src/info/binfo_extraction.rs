use std::{
    io::{self, ErrorKind},
    path::Path,
    process::Command,
    sync::LazyLock,
};

use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use semver::Version;

use super::parse_blender_ver;

struct InfoRegexes {
    ctime: Regex,
    cdate: Regex,
    build_hash: Regex,
    subversion: Regex,
    branch: Regex,
}
impl InfoRegexes {
    fn new() -> Self {
        Self {
            ctime: Regex::new(r"build commit time: (.*)").unwrap(),
            cdate: Regex::new(r"build commit date: (.*)").unwrap(),
            build_hash: Regex::new(r"build hash: (.*)").unwrap(),
            subversion: Regex::new(r"Blender (.*)").unwrap(),
            branch: Regex::new(r"build branch: (.*)").unwrap(),
        }
    }
}
static INFO_REGEXES: LazyLock<InfoRegexes> = LazyLock::new(InfoRegexes::new);

#[derive(Debug, Clone)]
pub struct CollectedInfo {
    pub commit_dt: Option<DateTime<Utc>>,
    pub build_hash: Option<String>,
    pub branch: Option<String>,
    pub subversion: Option<Version>,
    pub custom_name: Option<String>,
}

pub fn get_info_from_blender(executable: &Path) -> io::Result<CollectedInfo> {
    let binding = &mut Command::new(executable);
    let cmd = binding.arg("-v");

    let output = cmd.output()?;

    let text = match String::from_utf8(output.stdout) {
        Ok(t) => t,
        Err(e) => return Err(io::Error::new(ErrorKind::Unsupported, e)),
    };
    let commit_dt = {
        if let (Some(cd), Some(ct)) = (
            INFO_REGEXES.cdate.captures(&text),
            INFO_REGEXES.ctime.captures(&text),
        ) {
            if let (Some(d), Some(t)) = (cd.get(1), ct.get(1)) {
                let formatted = format!["{} {}", d.as_str(), t.as_str()];
                NaiveDateTime::parse_from_str(&formatted, "%F %H:%M")
                    .ok()
                    .map(|i| i.and_utc())
            } else {
                None
            }
        } else {
            None
        }
    };

    let build_hash = INFO_REGEXES
        .build_hash
        .captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let branch = INFO_REGEXES
        .branch
        .captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let (custom_name, subversion) = INFO_REGEXES
        .subversion
        .captures(&text)
        .and_then(|c| c.get(1))
        .and_then(|m| parse_blender_ver(m.as_str(), false).map(|v| (None, Some(v))))
        .or_else(|| {
            // Read the first line of stdout to parse the version
            text.lines()
                .next()
                .unwrap()
                .trim()
                .split_once(" ")
                .map(|(name, ver)| (Some(name.to_string()), parse_blender_ver(ver.trim(), false)))
        })
        .unwrap_or_default();

    Ok(CollectedInfo {
        commit_dt,
        build_hash,
        branch,
        subversion,
        custom_name,
    })
}
