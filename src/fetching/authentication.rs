use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// A struct holding proxy configuration settings
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProxyConfig {
    /// The hostname of the proxy server
    pub url: String,
    /// The username for the user
    pub user: String,
    /// The password for the user
    pub password: String,
}

impl Debug for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyConfig")
            .field("REDACTED", &"REDACTED")
            .finish()
    }
}

/// A struct holding GitHub authentication settings
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GithubAuthentication {
    /// The username of the GitHub account.
    pub user: String,
    /// The password or token for the GitHub account.
    pub token: String,
}

impl GithubAuthentication {
    /// Returns a new GithubAuthentication with the specified username and password.
    pub fn new(username: String, password: String) -> Self {
        Self {
            user: username,
            token: password,
        }
    }
}
