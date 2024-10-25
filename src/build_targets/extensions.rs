use std::env::consts::{ARCH, OS};

use crate::repos::{BuildEntry, RepoEntry};

/// File extension typically used for Linux targets.
pub const TARGET_LINUX_EXT: &str = "xz";
///  File extension typically used for Windows targets.
pub const TARGET_WINDOWS_EXT: &str = "zip";
/// File extension typically used for macOS targets.
pub const TARGET_MACOS_EXT: &str = "dmg";

/// Readable file types corresponding to different target operating systems.
pub const READABLE_FILETYPES: [&str; 3] = [TARGET_LINUX_EXT, TARGET_WINDOWS_EXT, TARGET_MACOS_EXT];

/// Retrieves the appropriate target setup based on the current system architecture and operating system.
///
/// If the platform is not supported, returns `None`.
pub fn get_target_setup() -> Option<(&'static str, &'static str, &'static str)> {
    let arch = match (ARCH, OS) {
        ("aarch64", _) => "arm64",
        ("x86_64", "windows") => "amd64",
        (x, _) => x,
    };

    match OS {
        "linux" => Some((OS, arch, TARGET_LINUX_EXT)),
        "macos" => Some(("darwin", arch, TARGET_MACOS_EXT)),
        "windows" => Some((OS, arch, TARGET_WINDOWS_EXT)),
        _ => None,
    }
}

/// Filters a list of repositories based on the target platform.
///
/// This function iterates over each repository and filters the build entries within it.
/// Build entries that don't match the target platform are removed.
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
