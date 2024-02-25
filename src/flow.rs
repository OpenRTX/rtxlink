use crate::cat;
use crate::fmp;

pub fn backup(dest_path: Option<String>) {
    // Default path is .
    let dest_path = match dest_path {
        Some(x) => x,
        _ => String::from("."),
    };
    let radio_name = cat::info();
    // Enumerate all the memories, dump each in a separate file
    let mem_list = fmp::meminfo();
    // Put the radio in file transfer mode and dump all the memories
    cat::ftm();
    for (i, mem) in mem_list.iter().enumerate() {
        let mut file_name: String = dest_path.clone();
        file_name.push_str(&radio_name);
        file_name.push_str("_");
        file_name.push_str(&mem.to_string());
        file_name.push_str(&chrono::offset::Local::now().format("_%d%m%Y")
                                                        .to_string());
        file_name.push_str(".bin");
        match fmp::dump(i, &mem, &file_name) {
            Err(why) => panic!("Error during radio backup: {}", why),
            Ok(_) => (),
        }
    }
}

pub fn restore(mem_index: Option<String>, src_path: Option<String>) {
    // Parse parameters
    let mem_index = mem_index.expect("Error: memory index not found!")
                             .parse::<usize>()
                             .expect("Error: invalid memory index!");
    let src_path = src_path.expect("Error: backup file not found!");
    let mem_list = fmp::meminfo();
    if mem_index > mem_list.len() {
        panic!("Error: memory index outsize range!");
    }
    fmp::flash(mem_index, &mem_list[mem_index], &src_path);
}
