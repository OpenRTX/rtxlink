use url::Url;
use std::sync::mpsc::Sender;

use crate::cat;
use crate::fmp;

pub fn backup(dest_path: Option<String>, progress: Option<&Sender<(usize, usize)>>) {
    // Default path is .
    let dest_path = match dest_path {
        Some(x) => x,
        _ => String::from("."),
    };
    // If it's a URI decode it to a path
    let dest_uri = Url::parse(&dest_path).unwrap();
    let dest_path = dest_uri.to_file_path().unwrap();
    let radio_name = cat::info();
    // Enumerate all the memories, dump each in a separate file
    let mem_list = fmp::meminfo();
    // Put the radio in file transfer mode and dump all the memories
    cat::ftm();
    for (i, mem) in mem_list.iter().enumerate() {
        let mut file_name = String::new();
        file_name.push_str(dest_path.to_str().unwrap());
        file_name.push_str(&radio_name);
        file_name.push_str("_");
        file_name.push_str(&mem.to_string());
        file_name.push_str(&chrono::offset::Local::now().format("_%d%m%Y")
                                                        .to_string());
        file_name.push_str(".bin");
        match fmp::dump(i, &mem, &file_name, progress) {
            Err(why) => panic!("Error while storing backup on {}: {}", file_name, why),
            Ok(x) => x,
        }
    }
}

pub fn restore(mem_index: Option<String>, src_path: Option<String>, progress: Option<&Sender<(usize, usize)>>) {
    // Parse parameters
    let mem_index = mem_index.expect("Error: memory index not found!")
                             .parse::<usize>()
                             .expect("Error: invalid memory index!");
    let src_path = src_path.expect("Error: backup file not found!");
    let mem_list = fmp::meminfo();
    if mem_index > mem_list.len() {
        panic!("Error: memory index outsize range!");
    }
    fmp::flash(mem_index, &mem_list[mem_index], &src_path, progress);
}
