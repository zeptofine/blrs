use std::env::consts::{ARCH, OS};

use crate::repos::{BuildEntry, RepoEntry};

pub const TARGET_LINUX_EXT: &str = "xz";
pub const TARGET_WINDOWS_EXT: &str = "zip";
pub const TARGET_MACOS_EXT: &str = "dmg";

pub const READABLE_FILETYPES: [&str; 3] = [TARGET_LINUX_EXT, TARGET_WINDOWS_EXT, TARGET_MACOS_EXT];

pub fn get_target_setup() -> Option<(&'static str, &'static str, &'static str)> {
    let arch = match ARCH {
        "aarch64" => "arm64",
        x => x,
    };

    match OS {
        "linux" => Some((OS, arch, TARGET_LINUX_EXT)),
        "macos" => Some(("darwin", arch, TARGET_MACOS_EXT)),
        "windows" => Some((OS, arch, TARGET_WINDOWS_EXT)),
        _ => None,
    }
}

pub fn filter_repos_by_target<V>(
    v: V,
    target: Option<(&'static str, &'static str, &'static str)>,
) -> Vec<RepoEntry>
where
    V: IntoIterator<Item = RepoEntry>,
{
    let target = target.unwrap_or(get_target_setup().unwrap());
    v.into_iter()
        .filter_map(|repo| {
            if let RepoEntry::Registered(r, vec) = repo {
                let new_build_entries: Vec<_> = vec
                    .into_iter()
                    .filter_map(|entry| {
                        if let BuildEntry::NotInstalled(variants) = entry {
                            let variants = variants.filter_target(target);
                            if variants.v.is_empty() {
                                None
                            } else {
                                Some(BuildEntry::NotInstalled(variants))
                            }
                        } else {
                            Some(entry)
                        }
                    })
                    .collect();

                if new_build_entries.is_empty() {
                    None
                } else {
                    Some(RepoEntry::Registered(r, new_build_entries))
                }
            } else {
                Some(repo)
            }
        })
        .collect()
}
