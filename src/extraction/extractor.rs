use std::{fs::File, path::Path};

use flate2::read::GzDecoder;
use zip::ZipArchive;

pub enum FileExtractor {
    Gz(GzDecoder<File>),
    Zip(ZipArchive<File>),
}

impl FileExtractor {
    fn from(p: &Path) -> Option<Self> {
        match p.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "tar.gz" => Some(Self::Gz(GzDecoder::new(File::open(p).ok()?))),
                "zip" => Some(Self::Zip(ZipArchive::new(File::open(p).ok()?).ok()?)),
                _ => None,
            },
            None => None,
        }
    }
}
