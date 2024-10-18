use std::fs::File;
use std::io::Read;
use std::path::Path;

// See https://docs.blender.org/manual/en/latest/files/blend/open_save.html#id8
#[derive(Default, Debug, Clone)]
pub enum CompressionType {
    Gzip, // used for < 3.0
    Zstd, // used for >= 3.0
    #[default]
    None, // used universally
}

#[derive(Debug, Clone, Default)]
pub struct BlendFileHeader {
    pub version: (u8, u8),
    pub compression_type: CompressionType,
}

const BYTE_REPRESENT_ZERO: u8 = b'0';

fn parse_header_version(nums: &[u8; 5]) -> (u8, u8) {
    let major = nums[0] - BYTE_REPRESENT_ZERO;
    let minor = nums[1] - BYTE_REPRESENT_ZERO;
    let patch = nums[2] - BYTE_REPRESENT_ZERO;
    (major, minor * 10 + patch)
}

fn read_basic_header(path: &Path) -> Result<[u8; 5], std::io::Error> {
    let mut file = File::open(path)?;

    let mut header_bytes = [0; 7];
    file.read_exact(&mut header_bytes)?;

    let b = &header_bytes;
    if [b"BLENDER", b"BULLETf"].contains(&b) {
        let mut version_bytes = [0; 5];
        file.read_exact(&mut version_bytes)?;
        Ok(version_bytes)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "The file header does not match blender's traditional header".to_string(),
        ))
    }
}

fn read_gzip_header(path: &Path) -> Result<[u8; 5], std::io::Error> {
    use flate2::read::GzDecoder;
    let mut file = File::open(path)?;
    let mut decoder = GzDecoder::new(&mut file);
    let mut header = [0; 7];
    decoder.read_exact(&mut header)?;

    let mut version_bytes = [0; 5];
    decoder.read_exact(&mut version_bytes)?;
    Ok(version_bytes)
}

fn read_zstd_header(path: &Path) -> Result<[u8; 5], std::io::Error> {
    use zstd::Decoder as zstdDecoder;
    let file = File::open(path)?;
    let mut header = [0; 7];

    let mut decoder = zstdDecoder::new(file)?;
    decoder.read_exact(&mut header)?;

    let mut version_bytes = [0; 5];
    decoder.read_exact(&mut version_bytes)?;
    Ok(version_bytes)
}

type BlendReadErr = (std::io::Error, std::io::Error, std::io::Error);

fn get_blendfile_header(path: &Path) -> Result<([u8; 5], CompressionType), BlendReadErr> {
    let b_e = match read_basic_header(path).map(|b| (b, CompressionType::None)) {
        Ok(v) => return Ok(v),
        Err(e) => e,
    };

    let g_e = match read_gzip_header(path).map(|b| (b, CompressionType::Gzip)) {
        Ok(v) => return Ok(v),
        Err(e) => e,
    };

    let z_e = match read_zstd_header(path).map(|b| (b, CompressionType::Zstd)) {
        Ok(v) => return Ok(v),
        Err(e) => e,
    };

    Err((b_e, g_e, z_e))
}
/// Tries to read the first 7 bytes of a file, to check if it is a blender file.
pub fn read_blendfile_header(path: &Path) -> Result<BlendFileHeader, BlendReadErr> {
    get_blendfile_header(path).map(|(b, c)| BlendFileHeader {
        version: parse_header_version(&b),
        compression_type: c,
    })
}
