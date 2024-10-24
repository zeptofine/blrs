use serde::{Deserialize, Serialize};

use crate::BasicBuildInfo;

#[cfg(feature = "reqwest")]
use reqwest::Url;

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBuild {
    pub link: String,
    pub basic: BasicBuildInfo,
    pub platform: Option<String>,
    pub architecture: Option<String>,
    pub file_extension: Option<String>,
}
impl std::fmt::Display for RemoteBuild {
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

pub struct ParseError;

impl RemoteBuild {
    pub fn string_with_link(&self) -> String {
        format!["{} - {:?}", self, self.link]
    }

    #[cfg(feature = "reqwest")]
    pub fn url(&self) -> Url {
        Url::parse(&self.link).unwrap()
    }
}
