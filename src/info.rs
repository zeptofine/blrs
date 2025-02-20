mod binfo_extraction;
mod blendfile_reader;
mod verbose_version;

/// This module provides functionality to extract, parse, and house build-related data from Blender builds.
pub mod build_info;
/// Module containing basic information about Blender builds.
pub mod launching;

pub use binfo_extraction::{get_info_from_blender, CollectedInfo};
pub use blendfile_reader::{read_blendfile_header, BlendFileHeader, CompressionType};
pub use build_info::{parse_blender_ver, BasicBuildInfo, LocalBuild, OLDVER_CUTOFF};
pub use verbose_version::VerboseVersion;
