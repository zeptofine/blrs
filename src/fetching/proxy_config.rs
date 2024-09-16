use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProxyConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}
