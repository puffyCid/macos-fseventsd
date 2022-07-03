use std::{env, error::Error, fs::OpenOptions, io::Write};

use csv;
use macos_fseventsd;

fn main() {
    println!("Starting FSEvents parser...");
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let path = &args[1];
        let files = macos_fseventsd::parser::fseventsd(path);
        parse_files(&files);
        return;
    } else {
        let files = macos_fseventsd::parser::get_fseventsd();
        parse_files(&files);
    }
}

fn parse_files(files: &Result<Vec<String>, std::io::Error>) {
    match files {
        Ok(results) => {
            println!("Going to parse {} files", results.len());

            let data = parse_data(&results);
            match data {
                Ok(()) => {}
                Err(e) => println!("Failed parse FsEvents: {:?}", e),
            }
        }
        Err(e) => {
            println!("Failed to get FSevents files {:?}", e);
            return;
        }
    };
}

fn parse_data(files: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_path("output.csv")?;
    let mut json_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("output.json")?;

    writer.write_record(&["Path", "Flags", "Node", "Event ID"])?;
    let mut full_data = Vec::new();

    for file in files {
        println!("Parsing file: {}", file);
        let data = macos_fseventsd::parser::decompress(&file);
        match data {
            Ok(results) => {
                let fsevents_data_results = macos_fseventsd::parser::parse_fsevents(&results);
                match fsevents_data_results {
                    Ok((_, mut data_results)) => {
                        for parsed in &data_results {
                            writer.write_record(&[
                                &parsed.path,
                                &parsed.flags,
                                &parsed.node.to_string(),
                                &parsed.event_id.to_string(),
                            ])?;
                        }
                        full_data.append(&mut data_results)
                    }
                    Err(error) => {
                        println!("Failed parsing FsEvent file {} - {:?}\n", file, error)
                    }
                }
            }
            Err(e) => {
                println!("Failed to decompress file {} {:?}\n", file, e);
                continue;
            }
        }
    }
    writer.flush()?;

    let serde_data = serde_json::to_string(&full_data)?;
    json_file.write_all(serde_data.as_bytes())?;
    println!("\nFinished parsing FsEvents data. Saved results to: output.csv and output.json");
    Ok(())
}
