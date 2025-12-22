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
    #[must_use]
    pub fn version(&self) -> Version {
        Version::new(self.version.0 as u64, self.version.1 as u64, 0)
    }
}

// https://projects.blender.org/blender/blender/src/branch/main/source/blender/blenloader_core/BLO_core_blend_header.hh
//
// Low level version 0: the header is 12 bytes long.
// 0-6:  'BLENDER'
// 7:    '-' for 8-byte pointers (#SmallBHead8) or '_' for 4-byte pointers (#BHead4)
// 8:    'v' for little endian or 'V' for big endian
// 9-11: 3 ASCII digits encoding #BLENDER_FILE_VERSION (e.g. '305' for Blender 3.5)
//
const _BLEND_FILE_FORMAT_VERSION_0: u8 = 12;
//
// Lower level version 1: the header is 17 bytes long.
// 0-6:   'BLENDER'
// 7-8:   size of the header in bytes encoded as ASCII digits (always '17' currently)
// 9:     always '-'
// 10-11: File version format as ASCII digits (always '01' currently)
// 12:    always 'v'
// 13-16: 4 ASCII digits encoding #BLENDER_FILE_VERSION (e.g. '0405' for Blender 4.5)
//
// With this header, #LargeBHead8 is always used.
//
const BLEND_FILE_FORMAT_VERSION_1: u8 = 17;

const _MIN_SIZEOFBLENDERHEADER: u8 = 12;
const _MAX_SIZEOFBLENDERHEADER: u8 = 17;

fn parse_<R: Read>(mut reader: R) -> Result<(u8, u8), std::io::Error> {
    // The magic BLENDER is already covered for us

    let mut next_two = [0; 2];
    reader.read_exact(&mut next_two)?;

    match next_two[0] {
        b'-' | b'_' => {
            // Version 0
            let mut version_bytes = [0; 3];
            reader.read_exact(&mut version_bytes)?;
            let major = version_bytes[0].wrapping_sub(b'0');
            let minor = version_bytes[1].wrapping_sub(b'0');
            Ok((major, minor))
        }
        b'1'..b'9' => {
            // Version 1
            let header_size_bytes = next_two;
            let header_size = (header_size_bytes[0].wrapping_sub(b'0') as u8) * 10
                + header_size_bytes[1].wrapping_sub(b'0') as u8;

            match header_size {
                BLEND_FILE_FORMAT_VERSION_1 => {
                    reader.read_exact(&mut [0])?; // skip the '-'

                    reader.read_exact(&mut [0; 2])?; // skip the version format

                    // let mut version_format_bytes = [0; 2];
                    // reader.read_exact(&mut version_format_bytes)?;
                    // let _version_format = (version_format_bytes[0].wrapping_sub(b'0') as u8) * 10
                    //     + version_format_bytes[1].wrapping_sub(b'0') as u8;

                    reader.read_exact(&mut [0])?; // skip the 'v'

                    let mut version_bytes = [0; 4];
                    reader.read_exact(&mut version_bytes)?;
                    let major = version_bytes[0].wrapping_sub(b'0') * 10
                        + version_bytes[1].wrapping_sub(b'0');
                    let minor = version_bytes[2].wrapping_sub(b'0') * 10
                        + version_bytes[3].wrapping_sub(b'0');
                    Ok((major, minor))
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unsupported blender header size".to_string(),
                    ))
                }
            }
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unsupported blender header format".to_string(),
        )),
    }
}

fn parse_with_guard<R: Read>(mut r: R) -> Result<(u8, u8), std::io::Error> {
    let mut magic = [0; 7];
    r.read_exact(&mut magic)?;
    if [b"BLENDER", b"BULLETf"].contains(&&magic) {
        parse_(r)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "The file header does not match blender's traditional header".to_string(),
        ))
    }
}

fn read_basic_header(path: &Path) -> Result<(u8, u8), std::io::Error> {
    let file = File::open(path)?;
    parse_with_guard(file)
}

#[cfg(feature = "compressed-blends")]
fn read_gzip_header(path: &Path) -> Result<(u8, u8), std::io::Error> {
    use flate2::read::GzDecoder;
    let mut file = File::open(path)?;
    let decoder = GzDecoder::new(&mut file);
    parse_with_guard(decoder)
}

#[cfg(feature = "compressed-blends")]
fn read_zstd_header(path: &Path) -> Result<(u8, u8), std::io::Error> {
    use zstd::Decoder as zstdDecoder;
    let file = File::open(path)?;
    let decoder = zstdDecoder::new(file)?;
    parse_with_guard(decoder)
}

type BlendReadErr = (std::io::Error, Option<(std::io::Error, std::io::Error)>);

fn get_blendfile_header(path: &Path) -> Result<((u8, u8), CompressionType), BlendReadErr> {
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
    get_blendfile_header(path).map(|(version, compression_type)| BlendFileHeader {
        version,
        compression_type,
    })
}
