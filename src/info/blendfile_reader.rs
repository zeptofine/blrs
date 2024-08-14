use std::fs::File;
use std::io::Read;
use std::path::Path;

// See https://docs.blender.org/manual/en/latest/files/blend/open_save.html#id8
#[derive(Default, Debug, Clone)]
pub enum CompressionType {
    #[cfg(feature = "gzip")]
    Gzip, // used for < 3.0
    #[cfg(feature = "zstd")]
    Zstd, // used for >= 3.0
    #[default]
    None, // used universally
}

#[derive(Debug, Clone, Default)]
pub struct BlendFileHeader {
    pub version: (u8, u8),
    pub compression_type: CompressionType,
}

const BYTE_REPRESENT_ZERO: u8 = b"0"[0];

fn parse_header_version(nums: &[u8; 5]) -> (u8, u8) {
    let major = nums[0] - BYTE_REPRESENT_ZERO;
    let minor = nums[1] - BYTE_REPRESENT_ZERO;
    let patch = nums[2] - BYTE_REPRESENT_ZERO;
    (major, minor * 10 + patch)
}

fn read_basic_header(path: &Path) -> Option<[u8; 5]> {
    let mut file = File::open(path).ok()?;

    let mut header_bytes = [0; 7];
    file.read_exact(&mut header_bytes).ok()?;

    let b = &header_bytes;
    if [b"BLENDER", b"BULLETf"].contains(&b) {
        let mut version_bytes = [0; 5];
        file.read_exact(&mut version_bytes).ok()?;
        Some(version_bytes)
    } else {
        None
    }
}

#[cfg(feature = "gzip")]
fn read_gzip_header(path: &Path) -> Option<[u8; 5]> {
    use flate2::read::GzDecoder;
    let mut file = File::open(path).ok()?;
    let mut decoder = GzDecoder::new(&mut file);
    let mut header = [0; 7];
    decoder.read_exact(&mut header).ok()?;
    println!["{:?}", header];
    let mut version_bytes = [0; 5];
    decoder.read_exact(&mut version_bytes).ok()?;
    Some(version_bytes)
}

#[cfg(feature = "zstd")]
fn read_zstd_header(path: &Path) -> Option<[u8; 5]> {
    use zstd::Decoder as zstdDecoder;
    let file = File::open(path).ok()?;
    let mut header = [0; 7];

    let mut decoder = zstdDecoder::new(file).ok()?;
    decoder.read_exact(&mut header).ok()?;
    println!("{:?}", header);
    let mut version_bytes = [0; 5];
    decoder.read_exact(&mut version_bytes).ok()?;
    Some(version_bytes)
}

fn get_blendfile_header(path: &Path) -> Option<([u8; 5], CompressionType)> {
    let h = read_basic_header(path).map(|b| (b, CompressionType::None));
    if h.is_some() {
        println!["No compression detected, assuming none"];
        return h;
    }
    #[cfg(feature = "gzip")]
    {
        let h = read_gzip_header(path).map(|b| (b, CompressionType::Gzip));
        if h.is_some() {
            println!["gzip blendfile detected"];
            return h;
        }
    }
    #[cfg(feature = "zstd")]
    {
        let h = read_zstd_header(path).map(|b| (b, CompressionType::Zstd));
        if h.is_some() {
            println!["zstd blendfile detected"];
            return h;
        }
    }
    None
}

fn read_blendfile_header(path: &Path) -> Option<BlendFileHeader> {
    get_blendfile_header(path).map(|(b, c)| BlendFileHeader {
        version: parse_header_version(&b),
        compression_type: c,
    })
}
