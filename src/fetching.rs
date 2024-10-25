/// Module containing functionality related to authentication.
pub mod authentication;

/// Module responsible for building repositories.
///
/// This module contains functions and types necessary for interacting with various repository services.
pub mod build_repository;

/// API schemas the project can recognize.
pub mod build_schemas;

/// Module containing functionality related to checksums, like comparing build and its checksum.
pub mod checksums;

/// Fetcher module for downloading external dependencies or resources via HTTP requests.
#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
pub mod fetcher;
mod remote_build;
mod request_builder;

pub use remote_build::RemoteBuild;

pub use request_builder::{random_ua, ProxyOptions, SerialProxyOptions};
