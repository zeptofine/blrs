#[cfg(feature = "reqwest")]
use reqwest::{Proxy, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct SerialProxyOptions {
    pub url: String,
    pub user: String,
    pub password: String,
}

#[cfg(feature = "reqwest")]
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

#[cfg(feature = "reqwest")]
pub struct ProxyOptions {
    pub url: Url,
    pub user: String,
    pub password: String,
}

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

#[cfg(feature = "reqwest")]
pub fn builder(user_agent: &str, proxy: Option<ProxyOptions>) -> reqwest::ClientBuilder {
    let mut r = reqwest::ClientBuilder::new().user_agent(user_agent);

    r = match proxy {
        None => r,
        Some(options) => r.proxy(
            Proxy::all(options.url)
                .unwrap()
                .basic_auth(&options.user, &options.password),
        ),
    };

    r
}
