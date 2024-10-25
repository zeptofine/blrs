use serde::{Deserialize, Serialize};

use crate::BasicBuildInfo;

#[cfg(feature = "reqwest")]
use reqwest::Url;

/// A struct representing a remote build.
///
/// This contains information about a build retrieved from a URL,
/// such as its basic build info and any additional platform-specific details.
#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBuild {
    /// The URL of the build.
    pub link: String,

    /// The basic information of the build.
    pub basic: BasicBuildInfo,

    /// The platform on which this build was executed (optional).
    pub platform: Option<String>,

    /// The architecture used for this build (optional).
    pub architecture: Option<String>,

    /// The file extension associated with this build (optional).
    pub file_extension: Option<String>,
}

impl std::fmt::Display for RemoteBuild {
    /// Formats the remote build as a string, including platform and architecture information.
    ///
    /// If no platform or architecture is provided, "unknown" and "null" are displayed respectively.
    /// The file extension is also included if available; otherwise, ".???".
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![
            f,
            "{} {} ({})",
            self.platform
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            self.architecture
                .clone()
                .unwrap_or_else(|| "null".to_string()),
            self.file_extension
                .clone()
                .unwrap_or_else(|| ".???".to_string()),
        ]
    }
}

impl RemoteBuild {
    /// Gets a string representation of the remote build including the link.
    pub fn string_with_link(&self) -> String {
        format!["{} - {:?}", self, self.link]
    }

    /// Turns the link into a Url.
    ///
    /// If the `reqwest` feature is enabled (which it should be for most uses), this will parse the link into a valid `Url`.
    #[cfg(feature = "reqwest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
    pub fn url(&self) -> Url {
        Url::parse(&self.link).unwrap()
    }
}
