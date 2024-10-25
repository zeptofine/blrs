use std::fs::File;
use std::io::Read;
use std::path::Path;

use semver::Version;

/// The compression type used to store a Blender file.
///
/// These types are used in the file header and determine how the file is compressed.
///
/// See <https://docs.blender.org/manual/en/latest/files/blend/open_save.html#id8>
#[derive(Default, Debug, Clone)]
pub enum CompressionType {
    /// Compressed with Gzip for versions of Blender before 3.0.
    Gzip,
    /// Compressed with Zstd for versions of Blender 3.0 or later.
    Zstd,
    /// No compression used; the file is stored in its raw form.
    #[default]
    None,
}

/// The header information for a Blender file.
///
/// This struct contains metadata about the file, including the version and compression type.
#[derive(Debug, Clone, Default)]
pub struct BlendFileHeader {
    /// The major and minor version numbers of the Blender software used to create this file.
    pub version: (u8, u8),
    /// The compression type used in the file header.
    pub compression_type: CompressionType,
}

impl BlendFileHeader {
    /// Returns a `Version` instance representing the Blender version number stored in the file header.
    ///
    /// This method creates a `Version` instance using the major and minor version numbers as the major, minor, and patch versions respectively.
    pub fn version(&self) -> Version {
        Version::new(self.version.0 as u64, self.version.1 as u64, 0)
    }
}

const BYTE_REPRESENT_ZERO: u8 = b'0';

fn parse_header_version(nums: &[u8; 3]) -> (u8, u8) {
    let major = nums[0] - BYTE_REPRESENT_ZERO;
    let minor = nums[1] - BYTE_REPRESENT_ZERO;
    let patch = nums[2] - BYTE_REPRESENT_ZERO;
    (major, minor * 10 + patch)
}

fn read_basic_header(path: &Path) -> Result<[u8; 3], std::io::Error> {
    let mut file = File::open(path)?;

    let mut header_bytes = [0; 7];
    file.read_exact(&mut header_bytes)?;

    let b = &header_bytes;
    if [b"BLENDER", b"BULLETf"].contains(&b) {
        file.read_exact(&mut [0; 2])?;
        let mut version_bytes = [0; 3];
        file.read_exact(&mut version_bytes)?;
        Ok(version_bytes)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "The file header does not match blender's traditional header".to_string(),
        ))
    }
}

#[cfg(feature = "compressed-blends")]
fn read_gzip_header(path: &Path) -> Result<[u8; 3], std::io::Error> {
    use flate2::read::GzDecoder;
    let mut file = File::open(path)?;
    let mut decoder = GzDecoder::new(&mut file);
    let mut header = [0; 9];
    decoder.read_exact(&mut header)?;

    let mut version_bytes = [0; 3];
    decoder.read_exact(&mut version_bytes)?;
    Ok(version_bytes)
}

#[cfg(feature = "compressed-blends")]
fn read_zstd_header(path: &Path) -> Result<[u8; 3], std::io::Error> {
    use zstd::Decoder as zstdDecoder;
    let file = File::open(path)?;
    let mut header = [0; 9];

    let mut decoder = zstdDecoder::new(file)?;
    decoder.read_exact(&mut header)?;

    let mut version_bytes = [0; 3];
    decoder.read_exact(&mut version_bytes)?;
    Ok(version_bytes)
}

type BlendReadErr = (std::io::Error, Option<(std::io::Error, std::io::Error)>);

fn get_blendfile_header(path: &Path) -> Result<([u8; 3], CompressionType), BlendReadErr> {
    let b_e = match read_basic_header(path).map(|b| (b, CompressionType::None)) {
        Ok(v) => return Ok(v),
        Err(e) => e,
    };

    #[cfg(not(feature = "compressed-blends"))]
    return Err((b_e, None));

    #[cfg(feature = "compressed-blends")]
    {
        let g_e = match read_gzip_header(path).map(|b| (b, CompressionType::Gzip)) {
            Ok(v) => return Ok(v),
            Err(e) => e,
        };

        let z_e = match read_zstd_header(path).map(|b| (b, CompressionType::Zstd)) {
            Ok(v) => return Ok(v),
            Err(e) => e,
        };

        Err((b_e, Some((g_e, z_e))))
    }
}

/// Tries to read the first 7 bytes of a file, to check if it is a blender file.
pub fn read_blendfile_header(path: &Path) -> Result<BlendFileHeader, BlendReadErr> {
    get_blendfile_header(path).map(|(b, c)| BlendFileHeader {
        version: parse_header_version(&b),
        compression_type: c,
    })
}
