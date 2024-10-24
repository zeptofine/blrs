use std::{
    collections::HashMap,
    env::consts::OS,
    path::{Path, PathBuf},
};

use super::LocalBuild;

#[derive(Clone, Debug, Default)]
pub enum BlendLaunchTarget {
    #[default]
    None,
    File(PathBuf),
    OpenLast,
    Custom(Vec<String>),
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
            BlendLaunchTarget::Custom(new_args) => {
                args = args.into_iter().chain(new_args).collect()
            }
        }

        args
    }
}

#[derive(Clone, Debug)]
pub enum OSLaunchTarget {
    Linux,
    Windows { no_console: bool },
    MacOS,
}

impl Default for OSLaunchTarget {
    #[inline]
    fn default() -> Self {
        Self::try_default().unwrap()
    }
}

impl OSLaunchTarget {
    pub fn try_default() -> Option<Self> {
        match OS {
            "windows" => Some(Self::Windows { no_console: true }),
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::MacOS),
            _ => None,
        }
    }

    pub fn exe_name(&self) -> &'static str {
        match self {
            OSLaunchTarget::Linux => "blender",
            OSLaunchTarget::Windows { no_console } => match no_console {
                true => "blender_launcher.exe",
                false => "blender.exe",
            },
            OSLaunchTarget::MacOS => "Blender/Blender.app",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GeneratedParams {
    pub exe: PathBuf,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

impl GeneratedParams {
    pub fn from_exe<P>(pth: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            exe: pth.into(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArgGenerationError {}

#[derive(Clone, Debug)]
pub struct LaunchArguments {
    pub file_target: BlendLaunchTarget,
    pub os_target: OSLaunchTarget,
    pub env: Option<HashMap<String, String>>,
}

impl LaunchArguments {
    pub fn file(file: BlendLaunchTarget) -> Self {
        LaunchArguments {
            file_target: file,
            os_target: OSLaunchTarget::try_default().unwrap(),
            env: None,
        }
    }

    /// Resolves the launching arguments and creates the params required to launch blender
    pub fn assemble(self, lb: &LocalBuild) -> Result<GeneratedParams, ArgGenerationError> {
        let blender = lb.folder.join(
            lb.info
                .custom_exe
                .clone()
                .unwrap_or(self.os_target.exe_name().to_string()),
        );

        let (executable, args) = match self.os_target {
            OSLaunchTarget::Linux => (blender, None),
            OSLaunchTarget::Windows { no_console: _ } => (blender, None),
            OSLaunchTarget::MacOS => {
                let mut args = vec![
                    "-W".to_string(),
                    "-n".to_string(),
                    blender.to_str().unwrap().to_string(),
                ];

                match self.file_target {
                    BlendLaunchTarget::None => {}
                    BlendLaunchTarget::File(_)
                    | BlendLaunchTarget::OpenLast
                    | BlendLaunchTarget::Custom(_) => {
                        args.push("--args".to_string());
                    }
                }

                (
                    which::which("open").unwrap_or(PathBuf::from("open")),
                    Some(args),
                )
            }
        };

        Ok(GeneratedParams {
            exe: executable,
            args: args
                .or(Some(vec![]))
                .map(|a| self.file_target.clone().transform(a))
                .filter(|v| !v.is_empty()),
            env: match (lb.info.custom_env.clone(), self.env) {
                (None, None) => None,
                (None, Some(e)) | (Some(e), None) => Some(e),
                (Some(cenv), Some(genv)) => {
                    let mut new_env = cenv.clone();
                    new_env.extend(genv);
                    Some(new_env)
                }
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::LazyLock, time::SystemTime};

    use chrono::DateTime;

    use crate::info::{
        build_info::{LocalBuildInfo, VerboseVersion},
        launching::{BlendLaunchTarget, GeneratedParams, LaunchArguments, OSLaunchTarget},
        BasicBuildInfo, LocalBuild,
    };
    const TEST_BUILD: LazyLock<LocalBuild> = LazyLock::new(|| LocalBuild {
        folder: PathBuf::from("blender/"),
        info: LocalBuildInfo {
            basic: BasicBuildInfo {
                ver: VerboseVersion::new(4, 3, 0, None, None, None),
                commit_dt: DateTime::from(SystemTime::now()),
            },
            is_favorited: false,
            custom_name: None,
            custom_exe: None,
            custom_env: None,
        },
    });

    #[test]
    fn test_launch_targets() {
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::None,
                os_target: OSLaunchTarget::Linux,
                env: None,
            }
            .assemble(&TEST_BUILD)
            .unwrap(),
            GeneratedParams::from_exe("blender/blender")
        ];
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::OpenLast,
                os_target: OSLaunchTarget::Linux,
                env: None,
            }
            .assemble(&TEST_BUILD)
            .unwrap(),
            GeneratedParams {
                exe: PathBuf::from("blender/blender"),
                args: Some(vec!["--open-last".to_string()]),
                env: None
            }
        ];
        // This assumes that blendfile.blend does not exist and therefore will stay relative
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::File(PathBuf::from("blendfile.blend")),
                os_target: OSLaunchTarget::Linux,
                env: None,
            }
            .assemble(&TEST_BUILD)
            .unwrap(),
            GeneratedParams {
                exe: PathBuf::from("blender/blender"),
                args: Some(vec!["blendfile.blend".to_string()]),
                env: None
            }
        ];
        assert_eq![
            LaunchArguments {
                file_target: BlendLaunchTarget::Custom(vec![
                    "-b".to_string(),
                    "-a".to_string(),
                    "file.blend".to_string()
                ]),
                os_target: OSLaunchTarget::Linux,
                env: None
            }
            .assemble(&TEST_BUILD)
            .unwrap(),
            GeneratedParams {
                exe: PathBuf::from("blender/blender"),
                args: Some(vec![
                    "-b".to_string(),
                    "-a".to_string(),
                    "file.blend".to_string()
                ]),
                env: None
            },
        ];
    }
}
