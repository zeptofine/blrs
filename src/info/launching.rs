use std::{env::consts::OS, path::PathBuf};

use super::LocalBlendBuild;

#[derive(Clone, Debug, Default)]
pub enum BlendLaunchTarget {
    #[default]
    None,
    File(PathBuf),
    OpenLast,
    Custom {
        before: Vec<String>,
        after: Vec<String>,
    },
}

impl BlendLaunchTarget {
    pub fn transform(self, mut args: Vec<String>) -> Vec<String> {
        match self {
            BlendLaunchTarget::None => {}
            BlendLaunchTarget::File(path) => args.push(
                path.canonicalize()
                    .unwrap_or(path)
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            BlendLaunchTarget::OpenLast => args.push("--open-last".to_string()),
            BlendLaunchTarget::Custom { before, after } => {
                args = before.into_iter().chain(args).chain(after).collect()
            }
        }

        args
    }
}

#[derive(Clone, Debug)]
pub enum OSLaunchTarget {
    Linux { nohup: bool },
    Windows { no_console: bool },
    MacOS,
}

impl OSLaunchTarget {
    fn try_default() -> Option<Self> {
        match OS {
            "windows" => Some(Self::Windows { no_console: false }),
            "linux" => Some(Self::Linux { nohup: false }),
            "macos" => Some(Self::MacOS),
            _ => None,
        }
    }

    fn exe_name(&self) -> &'static str {
        match self {
            OSLaunchTarget::Linux { nohup: _ } => "blender",
            OSLaunchTarget::Windows { no_console } => match no_console {
                true => "blender_launcher.exe",
                false => "blender.exe",
            },
            OSLaunchTarget::MacOS => "Blender/Blender.app",
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArgRetrievalError {}

#[derive(Clone, Debug)]
pub struct LaunchArguments {
    pub file_target: BlendLaunchTarget,
    pub os_target: OSLaunchTarget,
}

impl LaunchArguments {
    pub fn generate(&self, lb: &LocalBlendBuild) -> Result<Vec<String>, ArgRetrievalError> {
        let blender = lb.folder.join(
            lb.info
                .custom_exe
                .clone()
                .unwrap_or(self.os_target.exe_name().to_string()),
        );

        let blender_s = blender.to_str().unwrap().to_string();

        let mut args = match self.os_target {
            OSLaunchTarget::Linux { nohup } => match (nohup, which::which("nohup")) {
                (true, Ok(pth)) => vec![pth.to_str().unwrap().to_string(), blender_s],
                _ => vec![blender_s],
            },
            OSLaunchTarget::Windows { no_console: _ } => vec![blender_s],
            OSLaunchTarget::MacOS => vec![
                which::which("open")
                    .map(|p| p.to_str().unwrap().to_string())
                    .unwrap_or("open".to_string()),
                "-W".to_string(),
                "-n".to_string(),
                blender_s,
                "--args".to_string(),
            ],
        };

        args = self.file_target.clone().transform(args);
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::SystemTime};

    use chrono::DateTime;
    use semver::{BuildMetadata, Prerelease, Version};

    use crate::info::{
        build_info::LocalBuildInfo,
        launching::{BlendLaunchTarget, LaunchArguments, OSLaunchTarget},
        BasicBuildInfo, LocalBlendBuild,
    };

    #[test]
    fn test_create_args() {
        let test_build = LocalBlendBuild {
            folder: PathBuf::from("/path/to/blender_4.3.0-stable-abcdef"),
            info: LocalBuildInfo {
                info: BasicBuildInfo {
                    version: Version {
                        major: 4,
                        minor: 3,
                        patch: 0,
                        pre: Prerelease::EMPTY,
                        build: BuildMetadata::EMPTY,
                    },
                    commit_dt: DateTime::from(SystemTime::now()),
                },
                is_favorited: false,
                custom_name: None,
                custom_exe: None,
            },
        };

        println!["{:?}", test_build];

        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::Linux { nohup: false },
            }
            .generate(&test_build)
            .unwrap(),
            vec!["/path/to/blender_4.3.0-stable-abcdef/blender"]
        ];
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::Linux { nohup: true },
            }
            .generate(&test_build)
            .unwrap(),
            vec![
                "/usr/bin/nohup",
                "/path/to/blender_4.3.0-stable-abcdef/blender"
            ]
        ];
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::Windows { no_console: false },
            }
            .generate(&test_build)
            .unwrap(),
            vec!["/path/to/blender_4.3.0-stable-abcdef/blender.exe"]
        ];
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::Windows { no_console: true },
            }
            .generate(&test_build)
            .unwrap(),
            vec!["/path/to/blender_4.3.0-stable-abcdef/blender_launcher.exe"]
        ];
        #[cfg(not(target_os = "macos"))]
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::MacOS,
            }
            .generate(&test_build)
            .unwrap(),
            vec![
                "open",
                "-W",
                "-n",
                "/path/to/blender_4.3.0-stable-abcdef/Blender/Blender.app",
                "--args"
            ]
        ];
        #[cfg(target_os = "macos")]
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::MacOS,
            }
            .generate(&test_build)
            .unwrap(),
            vec![
                // I dont actually know where the macos open command is but this is where I think it is
                "/bin/open",
                "-W",
                "-n",
                "/path/to/blender_4.3.0-stable-abcdef/Blender/Blender.app",
                "--args"
            ]
        ];
    }
}
