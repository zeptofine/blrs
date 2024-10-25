#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! ### `blrs` is a crate designed to streamline the management and utilization of Blender builds.
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
pub mod config;
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

pub use config::{BLRSConfig, BLRSPaths};
pub use config::{DEFAULT_LIBRARY_FOLDER, DEFAULT_REPOS_FOLDER, PROJECT_DIRS};
pub use fetching::RemoteBuild;
pub use info::{BasicBuildInfo, LocalBuild};
