use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProxyConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

impl Debug for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyConfig")
            .field("REDACTED", &"REDACTED")
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]

pub struct GithubAuthentication {
    pub user: String,
    pub token: String,
}
