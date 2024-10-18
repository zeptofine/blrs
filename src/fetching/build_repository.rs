use std::sync::LazyLock;

use log::debug;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};

use super::{
    build_schemas::{builder_schema::BlenderBuildSchema, github::GithubRelease},
    fetcher::FetcherState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepoType {
    Blender,
    GithubAPI,
}

impl RepoType {
    pub fn try_serialize(&self, data: Vec<u8>) -> Result<Vec<BlenderBuildSchema>, FetchError> {
        match self {
            RepoType::Blender => match String::from_utf8(data) {
                Err(_) => Err(FetchError::InvalidResponse),
                Ok(s) => match serde_json::from_str(&s) {
                    Ok(lst) => Ok(lst),
                    Err(_) => {
                        debug!["failed to parse string: {:?}", s];

                        Err(FetchError::FailedToDeserialize)
                    }
                },
            },
            RepoType::GithubAPI => match String::from_utf8(data) {
                Err(_) => Err(FetchError::InvalidResponse),
                Ok(s) => match serde_json::from_str::<GithubRelease>(&s) {
                    Ok(release) => Ok(release.to_build_schemas()),
                    Err(_) => Err(FetchError::FailedToDeserialize),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildRepo {
    pub repo_id: String,
    pub url: String,
    pub nickname: String,
    pub repo_type: RepoType,
}

impl BuildRepo {
    pub fn url(&self) -> Url {
        Url::parse(&self.url).unwrap()
    }
}

pub static DEFAULT_REPOS: LazyLock<[BuildRepo; 3]> = LazyLock::new(|| {
    [
        BuildRepo {
            repo_id: "builder.blender.org.daily".to_string(),
            url: "https://builder.blender.org/download/daily/?format=json&v=1".to_string(),
            nickname: "daily".to_string(),
            repo_type: RepoType::Blender,
        },
        BuildRepo {
            repo_id: "builder.blender.org.experimental".to_string(),
            url: "https://builder.blender.org/download/experimental/?format=json&v=1".to_string(),
            nickname: "experimental".to_string(),
            repo_type: RepoType::Blender,
        },
        BuildRepo {
            repo_id: "builder.blender.org.patch".to_string(),
            url: "https://builder.blender.org/download/patch/?format=json&v=1".to_string(),
            nickname: "patch".to_string(),
            repo_type: RepoType::Blender,
        },
    ]
});

#[derive(Debug)]
pub enum FetchError {
    ReturnCode(StatusCode, Option<&'static str>),
    Reqwest(reqwest::Error),
    InvalidResponse,
    FailedToDeserialize,
    IoError(std::io::Error),
}

pub async fn fetch_repo(
    client: Client,

    repo: BuildRepo,
) -> Result<Vec<BlenderBuildSchema>, FetchError> {
    let url = repo.url();

    debug!["Using client {:?}", client];

    let mut state = FetcherState::new(client, url);

    loop {
        state = state.advance().await;

        match &state {
            FetcherState::Downloading {
                response: _,
                downloaded_bytes: _,
                total_bytes: _,
            } => {}
            _ => break,
        }
    }

    match state {
        FetcherState::Downloading {
            response: _,
            downloaded_bytes: _,
            total_bytes: _,
        }
        | FetcherState::Ready(_, _) => unreachable!(),
        FetcherState::Finished { response, bytes } => {
            if !response.status().is_success() {
                return Err(FetchError::ReturnCode(
                    response.status(),
                    response.status().canonical_reason(),
                ));
            }
            let bytes = bytes.read();
            repo.repo_type.try_serialize(bytes.clone())
        }
        FetcherState::Err(e) => Err(FetchError::Reqwest(e)),
    }
}
