use std::{fs::read_dir, path::PathBuf};

use macos_fseventsd;

#[test]
fn fseventd_local_test() {
    let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_location.push("tests/test_data/DLS2");
    let test_data = read_dir(test_location.display().to_string()).unwrap();
    let mut files: Vec<String> = Vec::new();
    for file_path in test_data {
        let data = file_path.unwrap();
        files.push(data.path().display().to_string())
    }

    for fsevent in files {
        println!("{}", fsevent);
        let data = macos_fseventsd::parser::decompress(&fsevent).unwrap();
        assert!(data.len() > 10);
        println!("file len: {}", data.len());

        let results = macos_fseventsd::parser::parse_fsevents(&data).unwrap();
        println!("{}", results.len());
    }
}

#[test]
fn fseventd_system_filelist_test() {
    let files = macos_fseventsd::parser::get_fseventsd().unwrap();
    assert!(files.len() > 3);
}

#[test]
fn fseventd_system_files_test() {
    let files = macos_fseventsd::parser::get_fseventsd().unwrap();
    assert!(files.len() > 3);

    for fsevent in files {
        //println!("{}", fsevent);
        let data = macos_fseventsd::parser::decompress(&fsevent).unwrap();
        assert!(data.len() > 10);
        //println!("file len: {}", data.len());

        let results = macos_fseventsd::parser::parse_fsevents(&data).unwrap();
        println!("{}", results.len());
        for parsed in results {
            println!("{}", parsed.path);
        }
    }
}
