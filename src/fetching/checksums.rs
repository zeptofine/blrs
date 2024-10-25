use std::{collections::HashMap, fs::File, io::Read, path::Path, string::FromUtf8Error};

use hex::ToHex;
use log::debug;
use semver::Version;
use sha2::{Digest, Sha256};

use super::build_schemas::BlenderBuildSchema;

/// A struct representing a pair of SHA256 checksums associated with a Blender build schema.
#[derive(Debug, Default)]
pub struct Sha256Pair {
    /// The SHA256 checksum for the Blender build.
    pub sha256: Option<BlenderBuildSchema>,
    /// The Blender build schema itself.
    pub build: Option<BlenderBuildSchema>,
}

///  Constructs a HashMap mapping Blender version strings to Sha256Pair structs.
pub fn get_sha256_pairs(lst: Vec<BlenderBuildSchema>) -> HashMap<Version, Sha256Pair> {
    let mut map: HashMap<Version, Sha256Pair> = HashMap::new();

    for schema in lst {
        let ver = schema.full_version_and_platform();

        let entry = map.remove(&ver);
        if schema.file_extension == "sha256" {
            map.insert(
                ver,
                Sha256Pair {
                    sha256: Some(schema),
                    build: match entry {
                        Some(e) => e.build,
                        None => None,
                    },
                },
            );
        } else {
            map.insert(
                ver,
                Sha256Pair {
                    sha256: match entry {
                        Some(e) => e.sha256,
                        None => None,
                    },
                    build: Some(schema),
                },
            );
        }
    }

    map
}

/// Enum representing possible errors during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// Error encountered while converting UTF-8 encoded bytes.
    FromUtf8(FromUtf8Error),
    /// I/O error occurred during file operations.
    Io(std::io::Error),
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::Io(value)
    }
}
impl From<FromUtf8Error> for ParseError {
    fn from(value: FromUtf8Error) -> Self {
        ParseError::FromUtf8(value)
    }
}

///  Calculates the SHA256 hash of a file.
pub fn generate_sha256<P>(file: P) -> Result<String, std::io::Error>
where
    P: AsRef<Path>,
{
    let mut hasher = Sha256::new();
    let mut file = File::open(file)?;

    let mut b = [0; 4096];

    loop {
        let bytes_read = file.read(&mut b)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&b[..bytes_read]);
    }

    Ok(hasher.finalize().to_vec().encode_hex::<String>())
}

///  Compares the SHA256 hash of a file with a given checksum.
pub fn verify_sha256<P1, P2>(sha256_file: P1, checked_file: P2) -> Result<bool, ParseError>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    debug!("reading sha256 file...");
    let sha_bytes = {
        let mut sha_file = File::open(sha256_file)?;
        let mut b = vec![];
        sha_file.read_to_end(&mut b)?;

        String::from_utf8(b)?
    };
    debug!("Finished reading sha256 file: {:?}", sha_bytes);

    debug!("Computing sha256...");
    let calculated_sha = generate_sha256(checked_file)?;

    debug!("Finished computing sha256: {:?}", calculated_sha);

    Ok(sha_bytes == calculated_sha)
}

// pub async fn test_sha256() {
//     use crate::fetching::{
//         builder_schema::get_sha256_pairs, checksums::verify_sha256, from_builder::read_builder_file,
//     };
//     let sha_is_valid = verify_sha256("/home/zeptofine/Downloads/blender-4.2.0-alpha+main-PR109522.f723782e3a8c-darwin.arm64-release.dmg.sha256", "/home/zeptofine/Downloads/blender-4.2.0-alpha+main-PR109522.f723782e3a8c-darwin.arm64-release.dmg");
//     println!["{:?}", sha_is_valid];

//     let lst = read_builder_file(PathBuf::from("builder.blender.org.json"))
//         .await
//         .unwrap();

//     println!["{:?}", lst];
//     println!["Sorting..."];

//     let pairs = get_sha256_pairs(lst);

//     println!["{:#?}", pairs];
// }
