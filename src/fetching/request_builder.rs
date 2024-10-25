#[cfg(feature = "reqwest")]
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// Proxy options able to be serialized.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SerialProxyOptions {
    /// The url of the proxy
    pub url: String,
    /// The username of the proxy
    pub user: String,
    /// The password of the proxy
    pub password: String,
}

#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
impl TryInto<ProxyOptions> for SerialProxyOptions {
    type Error = ();

    fn try_into(self) -> Result<ProxyOptions, Self::Error> {
        match Url::parse(&self.url) {
            Ok(url) => Ok(ProxyOptions {
                url,
                user: self.user,
                password: self.password,
            }),
            Err(_) => Err(()),
        }
    }
}

/// Options for configuring a proxy for requests.
#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
pub struct ProxyOptions {
    /// The url of the proxy
    pub url: Url,
    /// The username of the proxy
    pub user: String,
    /// The password of the proxy
    pub password: String,
}

/// Generates a random user-agent
pub fn random_ua() -> String {
    format![
        "{}/{}/{}-{}-{}",
        env!["CARGO_PKG_NAME"],
        env!["CARGO_PKG_VERSION"],
        std::env::consts::ARCH,
        std::env::consts::OS,
        uuid::Uuid::new_v4()
    ]
}
