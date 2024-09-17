use std::path::PathBuf;

use chrono::DateTime;
use reqwest::Url;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::info::parse_blender_ver;

use super::builder_schema::BlenderBuildSchema;

/// ! This assumes the tag name is SemVer Compatible

pub type GithubReleases = Vec<GithubRelease>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRelease {
    url: String,
    assets_url: String,
    upload_url: String,
    html_url: String,
    id: usize,
    tag_name: String,
    target_commitish: String,
    name: String,
    prerelease: bool,
    assets: Vec<GithubReleaseAsset>,
}

impl GithubRelease {
    pub fn to_build_schemas(self) -> Vec<BlenderBuildSchema> {
        let version = parse_blender_ver(&self.tag_name, false)
            .ok_or(())
            .unwrap_or(Version::parse("1.0.0").unwrap());
        let branch = if self.prerelease {
            "release"
        } else {
            "prerelease"
        }
        .to_string();

        self.assets
            .into_iter()
            .map(|asset| {
                let download_url = Url::parse(&asset.browser_download_url).unwrap();
                let filepath = PathBuf::from(download_url.path());
                let stem = filepath.file_stem().unwrap().to_str().unwrap().to_string();
                let extension = {
                    filepath
                        .clone()
                        .extension()
                        .unwrap_or(filepath.clone().file_stem().unwrap())
                        .to_str()
                        .unwrap()
                        .to_string()
                };
                let dt = DateTime::parse_from_rfc3339(&asset.updated_at)
                    .unwrap()
                    .to_utc();

                let mut platform = "unknown_platform";
                if stem.contains("linux") {
                    platform = "linux";
                }
                if stem.contains("windows") {
                    platform = "windows";
                }
                if stem.contains("darwin") {
                    platform = "darwin";
                }

                BlenderBuildSchema {
                    app: self.name.clone(),
                    url: asset.browser_download_url,
                    version: version.to_string(),
                    branch: branch.clone(),
                    patch: None,
                    hash: "ffffffff".to_string(),
                    platform: platform.to_string(),
                    architecture: "unknown_arch".to_string(),
                    file_mtime: dt.timestamp() as usize,
                    file_name: stem,
                    file_size: asset.size,
                    file_extension: extension,
                    release_cycle: branch.clone(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseAsset {
    url: String,
    id: usize,
    name: String,
    content_type: String,
    size: usize,
    created_at: String,
    updated_at: String,
    browser_download_url: String,
}
