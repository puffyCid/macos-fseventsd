//! Parse macOS FsEvent data
//!
//! Provides a library to decompress and parse FsEvent files.

use crate::fsevents::FsEvents;
use flate2::read::MultiGzDecoder;
use std::{
    fs::{metadata, read_dir, File},
    io::{Error, ErrorKind, Read},
};

/// Decompress gzip compressed files
pub fn decompress(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut open = File::open(path)?;
    let meta = open.metadata()?;
    if !meta.is_file() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Not a file: {}", path),
        ));
    }
    let mut buffer = Vec::new();
    open.read_to_end(&mut buffer)?;
    let mut data = MultiGzDecoder::new(&buffer[..]);

    let mut decompress_data = Vec::new();
    data.read_to_end(&mut decompress_data)?;

    Ok(decompress_data)
}

/// Get FsEvents data from decompressed file
pub fn parse_fsevents(data: &[u8]) -> nom::IResult<&[u8], Vec<FsEvents>> {
    FsEvents::fsevents_data(data)
}

/// Get FsEvents files at default path
pub fn get_fseventsd() -> Result<Vec<String>, std::io::Error> {
    const CURRENT_PATH: &str = "/System/Volumes/Data/.fseventsd/";
    fseventsd(CURRENT_PATH)
}

/// Get FsEvents files at old path
pub fn get_fseventsd_legacy() -> Result<Vec<String>, std::io::Error> {
    const OLD_PATH: &str = "/.fseventsd";
    fseventsd(OLD_PATH)
}

/// Get list of files in a directory
pub fn fseventsd(directory: &str) -> Result<Vec<String>, std::io::Error> {
    if metadata(directory).is_err() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Not a directory: {}", directory),
        ));
    }
    let dir = read_dir(directory)?;
    let mut files: Vec<String> = Vec::new();

    // read all files under fsevents directory
    // Skip fseventsd-uuid because it is not a fsevents file
    for file_path in dir {
        let data = file_path?;
        if data.file_name() == "fseventsd-uuid" {
            continue;
        }
        files.push(data.path().display().to_string())
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use crate::parser::{decompress, fseventsd, get_fseventsd, parse_fsevents};
    use std::path::PathBuf;

    #[test]
    fn test_get_fseventsd() {
        let files = get_fseventsd().unwrap();
        assert!(files.len() > 3);
    }

    #[test]
    fn test_decompress() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/DLS2/0000000000027d79");
        let test_path: &str = &test_location.display().to_string();
        let files = decompress(test_path).unwrap();
        assert!(files.len() == 78970);
    }

    #[test]
    fn test_fseventsd() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/DLS2/");
        let files = fseventsd(&test_location.display().to_string()).unwrap();
        assert!(files.len() == 2)
    }

    #[test]
    fn test_parse_fsevents() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/DLS2/0000000000027d79");
        let test_path: &str = &test_location.display().to_string();
        let files = decompress(test_path).unwrap();
        let (_, results) = parse_fsevents(&files).unwrap();
        assert!(results.len() == 736)
    }

    #[test]
    #[should_panic]
    fn test_malformed() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/Malformed/malformed");
        let test_path: &str = &test_location.display().to_string();
        let files = decompress(test_path).unwrap();
        let _results = parse_fsevents(&files).unwrap();
    }

    #[test]
    fn test_parse_fsevents_version1() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/DLS1/0000000000027d7a");
        let test_path: &str = &test_location.display().to_string();
        let files = decompress(test_path).unwrap();
        let (_, results) = parse_fsevents(&files).unwrap();

        assert!(results.len() == 2);
        assert!(results[0].path == "/.fseventsd/sl-compat");
        assert!(results[0].event_id == 163194);
        assert!(results[0].flags == "IsDirectory");
        assert!(results[0].node == 0);
    }
}
