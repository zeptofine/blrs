use reqwest::{Client, Response, Url};

/// A helper method for [FetcherState::new].
#[inline]
pub fn fetch(client: Client, url: Url) -> FetcherState {
    FetcherState::new(client, url)
}
/// Fetcher state machine.
///
/// This enum represents the different states that the fetcher can be in.
/// It is used to manage the fetch process and handle any errors that may occur.
/// This variation only keeps the last chunk of data in its storage.
pub enum FetchStreamerState {
    /// Initial ready state, where the client and URL are specified.
    Ready(Client, Url),

    /// Downloading state, where data is being fetched from the server.
    Downloading {
        /// The HTTP response object.
        response: Response,

        /// The last chunk of bytes that was received.
        last_chunk: Vec<u8>,
    },

    /// Finished state, where the fetch process is complete.
    Finished {
        /// The HTTP response object.
        response: Response,
    },

    /// Error state, where an error occurred during the fetch process.
    Err(reqwest::Error),
}

impl FetchStreamerState {
    /// Creates a new `FetchStreamerState` instance in the ready state.
    #[inline]
    pub fn new(client: Client, url: Url) -> Self {
        Self::Ready(client, url)
    }

    /// Advances the fetcher to the next state based on the current state.
    ///
    /// This method is used to manage the fetch process and handle any errors that
    /// may occur. It returns a new [`FetchStreamerState`] instance with the updated
    /// state.
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

/// Fetcher state machine.
///
/// This enum represents the different states that the fetcher can be in.
/// It is used to manage the fetch process and handle any errors that may occur.
#[derive(Debug)]
pub enum FetcherState {
    /// Initial ready state, where the client and URL are specified.
    Ready(Client, Url),

    /// Downloading state, where data is being fetched from the server.
    Downloading {
        /// The HTTP response object.
        response: Response,

        /// The downloaded bytes so far
        downloaded_bytes: Vec<u8>,

        /// The total size of the file (optional).
        total_bytes: Option<u64>,
    },

    /// Finished state, where the fetch process is complete.
    Finished {
        /// The HTTP response object.
        response: Response,

        /// The downloaded bytes
        bytes: Vec<u8>,
    },

    /// Error state, where an error occurred during the fetch process.
    Err(reqwest::Error),
}

impl FetcherState {
    /// Creates a new `FetcherState` instance in the ready state.
    #[inline]
    pub fn new(client: Client, url: Url) -> Self {
        Self::Ready(client, url)
    }

    /// Advances the fetcher to the next state based on the current state.
    ///
    /// This method is used to manage the fetch process and handle any errors that
    /// may occur. It returns a new [`FetcherState`] instance with the updated
    /// state.
    pub async fn advance(self) -> Self {
        match self {
            Self::Ready(client, url) => {
                let response = client.get(url).send().await;
                match response {
                    Ok(response) => Self::Downloading {
                        total_bytes: response.content_length(),
                        response,
                        downloaded_bytes: vec![],
                    },
                    Err(e) => Self::Err(e),
                }
            }
            Self::Downloading {
                mut response,
                mut downloaded_bytes,
                total_bytes,
            } => match response.chunk().await {
                Ok(Some(bytes)) => {
                    downloaded_bytes.extend(bytes);

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
