use serde::{Deserialize, Serialize};

use crate::BasicBuildInfo;

use reqwest::Url;

#[derive(PartialEq, PartialOrd, Debug, Clone, Serialize, Deserialize)]
pub struct RemoteBuild {
    pub link: String,
    pub basic: BasicBuildInfo,
}

pub struct ParseError;

impl RemoteBuild {
    pub fn parse(link: String, basic: BasicBuildInfo) -> Result<Self, ParseError> {
        match Url::parse(&link) {
            // Make sure `link` is a valid URL
            Ok(_url) => Ok(Self { link, basic }),
            Err(_) => Err(ParseError),
        }
    }

    pub fn url(&self) -> Url {
        Url::parse(&self.link).unwrap()
    }
}
