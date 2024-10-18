use std::sync::Arc;

use reqwest::{Client, Response, Url};

use parking_lot::RwLock;

#[inline]
pub fn fetch(client: Client, url: Url) -> FetcherState {
    FetcherState::new(client, url)
}
pub enum FetchStreamerState {
    Ready(Client, Url),
    Downloading {
        response: Response,
        last_chunk: Vec<u8>,
    },
    Finished {
        response: Response,
    },
    Err(reqwest::Error),
}

impl FetchStreamerState {
    #[inline]
    pub fn new(client: Client, url: Url) -> Self {
        Self::Ready(client, url)
    }

    pub async fn advance(self) -> Self {
        match self {
            Self::Ready(client, url) => {
                let response = client.get(url).send().await;
                match response {
                    Ok(response) => Self::Downloading {
                        response,
                        last_chunk: vec![],
                    },
                    Err(e) => Self::Err(e),
                }
            }
            Self::Downloading {
                mut response,
                mut last_chunk,
            } => match response.chunk().await {
                Ok(Some(bytes)) => {
                    last_chunk.clear();
                    last_chunk.extend(bytes);

                    Self::Downloading {
                        response,
                        last_chunk,
                    }
                }
                Ok(None) => Self::Finished { response },
                Err(e) => Self::Err(e),
            },

            x => x,
        }
    }
}

pub enum FetcherState {
    Ready(Client, Url),
    Downloading {
        response: Response,
        downloaded_bytes: Arc<RwLock<Vec<u8>>>,
        total_bytes: Option<u64>,
    },
    Finished {
        response: Response,
        bytes: Arc<RwLock<Vec<u8>>>,
    },
    Err(reqwest::Error),
}

impl FetcherState {
    #[inline]
    pub fn new(client: Client, url: Url) -> Self {
        Self::Ready(client, url)
    }

    pub async fn advance(self) -> Self {
        match self {
            Self::Ready(client, url) => {
                let response = client.get(url).send().await;
                match response {
                    Ok(response) => Self::Downloading {
                        total_bytes: response.content_length(),
                        response,
                        downloaded_bytes: Arc::new(RwLock::new(vec![])),
                    },
                    Err(e) => Self::Err(e),
                }
            }
            Self::Downloading {
                mut response,
                downloaded_bytes,
                total_bytes,
            } => match response.chunk().await {
                Ok(Some(bytes)) => {
                    {
                        let mut b = downloaded_bytes.write();

                        b.extend(bytes.clone());
                    }

                    Self::Downloading {
                        response,
                        downloaded_bytes,
                        total_bytes,
                    }
                }
                Ok(None) => Self::Finished {
                    response,
                    bytes: downloaded_bytes,
                },
                Err(e) => Self::Err(e),
            },
            x => x,
        }
    }
}
