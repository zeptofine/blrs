use std::{collections::HashMap, env::consts::OS, path::PathBuf};

use super::LocalBuild;

/// An enum specifying stuff fed to blender when built.
#[derive(Clone, Debug, Default)]
pub enum BlendLaunchTarget {
    /// No target specified.
    #[default]
    None,
    /// Open a specific blend file.
    File(PathBuf),
    /// Open the last blend file.
    OpenLast,
    /// Launch Blender with custom arguments.
    Custom(Vec<String>),
}

impl BlendLaunchTarget {
    /// Modifies the provided argument vector based on the launch target.
    pub fn transform(&self, args: &mut Vec<String>) {
        match self {
            BlendLaunchTarget::None => {}
            BlendLaunchTarget::File(path) => args.push(match path.canonicalize() {
                Ok(p) => p.to_str().unwrap().to_string(),
                Err(_) => path.to_str().unwrap().to_string(),
            }),
            BlendLaunchTarget::OpenLast => args.push("--open-last".to_string()),
            BlendLaunchTarget::Custom(new_args) => args.extend(new_args.iter().cloned()),
        }
    }
}

/// An enum specifying the target OS and its specific launch configuration.
#[derive(Clone, Debug)]
pub enum OSLaunchTarget {
    /// Linux environment.
    Linux,
    /// Windows environment with optional console flag.
    Windows {
        /// Whether to launch Blender without a console window. This is relevant for GUI-based launches.
        no_console: bool,
    },
    /// macOS environment.
    MacOS,
}

impl Default for OSLaunchTarget {
    #[inline]
    fn default() -> Self {
        Self::try_default().unwrap()
    }
}

impl OSLaunchTarget {
    /// Attempts to determine the default launch target based on the current OS.
    pub fn try_default() -> Option<Self> {
        match OS {
            "windows" => Some(Self::Windows { no_console: false }),
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::MacOS),
            _ => None,
        }
    }

    /// Returns the appropriate executable name for the current OS target.
    pub fn exe_name(&self) -> &'static str {
        match self {
            OSLaunchTarget::Linux => "blender",
            OSLaunchTarget::Windows { no_console } => match no_console {
                true => "blender-launcher.exe",
                false => "blender.exe",
            },
            OSLaunchTarget::MacOS => "Blender/Blender.app",
        }
    }
}

/// Struct holding parameters required to launch Blender.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GeneratedParams {
    /// Executable path
    pub exe: PathBuf,

    /// command arguments
    pub args: Option<Vec<String>>,

    /// environment variables
    pub env: Option<HashMap<String, String>>,
}

impl GeneratedParams {
    /// Creates a new `GeneratedParams` instance with only the executable path.
    pub fn from_exe<P>(pth: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            exe: pth.into(),
            ..Default::default()
        }
    }

    /// Extends the current arguments with the given args
    pub fn extend_args(&mut self, args: Vec<String>) {
        match &mut self.args {
            Some(self_args) => {
                self_args.extend(args);
            }
            None => {
                self.args = Some(args);
            }
        }
    }
}

impl From<GeneratedParams> for std::process::Command {
    fn from(value: GeneratedParams) -> Self {
        let GeneratedParams { exe, args, env } = value;
        let mut command = std::process::Command::new(exe);
        command
            .args(
                args.unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<String>>(),
            )
            .envs(env.clone().unwrap_or_default());
        command
    }
}

#[derive(Clone, Debug)]
/// Errors related to generating parameters.
pub enum ArgGenerationError {}

/// Struct holding the arguments required to launch Blender with specific configurations.
#[derive(Clone, Debug)]
pub struct LaunchArguments {
    /// Specifies the file to open in Blender or a custom command for launching.
    pub file_target: BlendLaunchTarget,

    /// Determines the target operating system and its associated launch configuration.
    pub os_target: OSLaunchTarget,

    /// Optional environment variables to be passed to Blender.
    pub env: Option<HashMap<String, String>>,
}

impl LaunchArguments {
    /// Creates a new `LaunchArguments` instance with only the file target specified as `None`.
    pub fn file(file: BlendLaunchTarget) -> Self {
        LaunchArguments {
            file_target: file,
            os_target: OSLaunchTarget::try_default().unwrap(),
            env: None,
        }
    }

    /// Resolves the launching arguments and creates the params required to launch blender
    pub fn assemble(self, lb: &LocalBuild) -> Result<GeneratedParams, ArgGenerationError> {
        let blender = match &lb.info.custom_exe {
            Some(e) => lb.folder.join(e),
            None => lb.folder.join(self.os_target.exe_name()),
        };

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

                (PathBuf::from("open"), Some(args))
            }
        };

        Ok(GeneratedParams {
            exe: executable,
            args: args
                .or(Some(vec![]))
                .map(|mut a| {
                    self.file_target.transform(&mut a);
                    a
                })
                .filter(|v| !v.is_empty()),
            env: match (lb.info.custom_env.clone(), self.env) {
                (None, None) => None,
                (None, Some(e)) | (Some(e), None) => Some(e),
                (Some(mut new_env), Some(genv)) => {
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
        build_info::LocalBuildInfo,
        launching::{BlendLaunchTarget, GeneratedParams, LaunchArguments, OSLaunchTarget},
        BasicBuildInfo, LocalBuild, VerboseVersion,
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
