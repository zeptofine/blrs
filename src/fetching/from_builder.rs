use std::path::PathBuf;

use async_std::io::ReadExt;
use reqwest::{Client, Url};

use super::{builder_schema::RemoteBuildSchema, fetcher::FetcherState};

type BuilderList = Vec<RemoteBuildSchema>;

#[derive(Debug)]
pub enum FetchError {
    Reqwest(reqwest::Error),
    InvalidResponse,
    FailedToDeserialize,
    IoError(std::io::Error),
}

#[inline]
fn to_builder_list(bytes: Vec<u8>) -> Result<BuilderList, FetchError> {
    match String::from_utf8(bytes) {
        Err(_) => Err(FetchError::InvalidResponse),
        Ok(s) => match serde_json::from_str(&s) {
            Ok(lst) => Ok(lst),
            Err(_) => Err(FetchError::FailedToDeserialize),
        },
    }
}

pub async fn fetch_builds_from_builder(client: Client, p: Url) -> Result<BuilderList, FetchError> {
    let mut state = FetcherState::new(client, p);

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
        FetcherState::Finished { response: _, bytes } => {
            let bytes = bytes.read();
            to_builder_list(bytes.clone())
        }
        FetcherState::Err(e) => Err(FetchError::Reqwest(e)),
    }
}

pub async fn read_builder_file(path: PathBuf) -> Result<BuilderList, FetchError> {
    let mut file = match async_std::fs::File::open(path).await {
        Ok(f) => f,
        Err(e) => return Err(FetchError::IoError(e)),
    };
    let mut bytes = vec![];
    if let Err(e) = file.read_to_end(&mut bytes).await {
        return Err(FetchError::IoError(e));
    }

    to_builder_list(bytes)
}

pub fn default_url() -> Url {
    Url::parse("https://builder.blender.org/download/daily/?format=json&v=1").unwrap()
}
