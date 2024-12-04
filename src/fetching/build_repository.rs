use std::sync::LazyLock;

use log::debug;

use serde::{Deserialize, Serialize};

#[cfg(feature = "reqwest")]
use reqwest::{Client, StatusCode, Url};

use super::build_schemas::{
    BlenderBuildSchema,
    // github::GithubRelease
};

/// Enum representing the different types of repositories that can be fetched.
///
/// Each variant corresponds to a specific repository type and has its own method for
/// deserializing the response data into a list of `BlenderBuildSchema` objects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepoType {
    /// The Blender repository type. Data is expected to be in JSON format.
    Blender,
    // /// The GitHub API repository type. Data is also expected to be in JSON format and
    // /// represents a single release. It is then converted into a list of `BlenderBuildSchema`
    // /// objects using the `to_build_schemas` method.
    // GithubAPI,
}

impl RepoType {
    /// Attempts to deserialize the given response data into a list of `BlenderBuildSchema`
    /// objects, depending on the type of repository specified.
    ///
    /// Returns an error if deserialization fails for any reason, or if the response data is
    /// invalid (e.g. not in JSON format).
    pub fn try_serialize(&self, data: Vec<u8>) -> Result<Vec<BlenderBuildSchema>, FetchError> {
        match self {
            RepoType::Blender => match String::from_utf8(data) {
                Err(_) => Err(FetchError::InvalidResponse),
                Ok(s) => match serde_json::from_str(&s) {
                    Ok(lst) => Ok(lst),
                    Err(e) => {
                        debug!["failed to parse string: {:?}", s];

                        Err(FetchError::FailedToDeserialize(e))
                    }
                },
            },
            // RepoType::GithubAPI => match String::from_utf8(data) {
            //     Err(_) => Err(FetchError::InvalidResponse),
            //     Ok(s) => match serde_json::from_str::<GithubRelease>(&s) {
            //         Ok(release) => Ok(release.to_build_schemas()),
            //         Err(_) => Err(FetchError::FailedToDeserialize),
            //     },
            // },
        }
    }
}
/// Represents a build repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildRepo {
    /// A unique identifier for the repository.
    pub repo_id: String,
    /// The URL of the repository.
    pub url: String,
    /// A nickname for the repository.
    pub nickname: String,
    /// The type of repository (Blender or GithubAPI).
    pub repo_type: RepoType,
}

impl BuildRepo {
    /// Turns the link into a Url.
    ///
    /// If the `reqwest` feature is enabled (which it should be for most uses), this will parse the link into a valid `Url`.
    #[cfg(feature = "reqwest")]
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
    pub fn url(&self) -> Url {
        Url::parse(&self.url).unwrap()
    }
}

/// A list of default build repositories. They are representations of the official blender builder API.
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

/// Errors that can occur when fetching data from a repository.
#[derive(Debug)]
pub enum FetchError {
    /// An HTTP return code that indicates an error.
    #[cfg(feature = "reqwest")]
    ReturnCode(StatusCode, Option<&'static str>),
    /// An error returned by the `reqwest` library.
    #[cfg(feature = "reqwest")]
    Reqwest(reqwest::Error),
    /// An invalid response from the server.
    InvalidResponse,
    /// Failed to deserialize the response into readable format.
    FailedToDeserialize(serde_json::Error),
    /// There was an IO error when fetching.
    IoError(std::io::Error),
}

#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
/// Fetches data from a build repository using the provided client.
pub async fn fetch_repo(
    client: Client,
    repo: BuildRepo,
) -> Result<Vec<BlenderBuildSchema>, FetchError> {
    use super::fetcher::FetcherState;
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
