#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::pedantic, clippy::style)]
#![warn(unused_import_braces, unused_imports)]
#![allow(clippy::unused_self, clippy::unused_async)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#![allow(clippy::missing_panics_doc)]

//! ### `blrs` is a crate designed to streamline the management and utilization of Blender builds.
//!
//! I built this crate to be the backbone of my own build managing program, `blrs-cli`.
//! I intend to use it for a GUI in the future aswell.
//!
//! It provides tools for configuring BLRS settings,
//! fetching build artifacts from remote repositories, and interacting with local and remote build information.
//! This crate helps users effectively categorize
//! and utilize various Blender builds across diverse environments. Use cases include:
//!
//! * **Blender Build Management:** Easily download, manage, and organize Blender builds from different sources.
//! * **Build Comparison and Selection:** Efficiently compare the characteristics of various builds to select the most suitable option for a specific project or purpose.
//!
//! Selectable Features
//! ---

#![doc = document_features::document_features!()]

/// BLRS level configuration settings.
#[cfg(feature = "config")]
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
pub mod config;

/// Path information for blrs-managed builds and blrs itself.
pub mod paths;

/// Utilities and methods for downloading artifacts.
pub mod fetching;
/// Collections to describe local and remote Blender builds.
pub mod info;

/// Collections to categorize build repositories.
pub mod repos;
/// Methods for grouping and filtering builds.
pub mod search;

/// Methods for filtering repos based on the build target.
pub mod build_targets;

#[cfg(feature = "config")]
pub use config::BLRSConfig;

pub use paths::{BLRSPaths, DEFAULT_LIBRARY_FOLDER, DEFAULT_REPOS_FOLDER, PROJECT_DIRS};

pub use fetching::RemoteBuild;
pub use info::{BasicBuildInfo, LocalBuild};
