use std::env;
use std::process;
use text_colorizer::*;

mod slip;
mod link;
mod cat;
mod fmp;
mod dat;

/// Print usage information of this tool
fn print_usage(cmd: &String) {
    eprintln!("{}: OpenRTX Communication Protocol", "rtxlink".yellow());
    eprintln!("{}: invalid parameters", "Error".red().bold());
    eprintln!("Usage: {cmd} SERIALPORT COMMAND [DATA_0..DATA_N]");
    eprintln!("commands:");
    eprintln!(" info                      Get device info");
    eprintln!(" freqrx                    Print receive frequency");
    eprintln!(" freqtx                    Print transmit frequency");
    eprintln!(" freqrx FREQ_MHZ           Set the receive frequency");
    eprintln!(" freqtx FREQ_MHZ           Set the transmit frequency");
    eprintln!(" backup                    Read the device flash and save it to flash_dump.bin");
    eprintln!(" restore MEM_IDX FILE      Write an image to the device flash");
    process::exit(1);
}

/// Print info about the target OpenRTX platform
fn print_info(serial_port: &str) {
    println!("Radio identifier: {}", cat::info(serial_port));
    let memlist = fmp::meminfo(serial_port);
    println!("Available memories:");
    let mut i: usize = 0;
    for mem in memlist {
        println!("[{}]: {:?}", i, mem);
        i += 1;
    };
}

pub fn backup(serial_port: &str) {
    let radio_name = cat::info(serial_port);
    // Enumerate all the memories, dump each in a separate file
    let mem_list = fmp::meminfo(serial_port);
    // Put the radio in file transfer mode and dump all the memories
    cat::ftm(serial_port);
    for (i, mem) in mem_list.iter().enumerate() {
        let mut file_name: String = String::from("");
        file_name.push_str(&radio_name);
        file_name.push_str("_");
        file_name.push_str(&mem.to_string());
        file_name.push_str(&chrono::offset::Local::now().format("_%d%m%Y")
                                                        .to_string());
        file_name.push_str(".bin");
        match fmp::dump(serial_port, i, &mem, &file_name) {
            Err(why) => panic!("Error during radio backup: {}", why),
            Ok(_) => (),
        }
    }
}

pub fn restore(serial_port: &str, mem_index: Option<String>, backup_path: Option<String>) {
    // Parse parameters
    let mem_index = mem_index.expect("Error: memory index not found!")
                             .parse::<usize>()
                             .expect("Error: invalid memory index!");
    let backup_path = backup_path.expect("Error: backup file not found!");
    let mem_list = fmp::meminfo(serial_port);
    if mem_index > mem_list.len() {
        panic!("Error: memory index outsize range!");
    }
    fmp::flash(serial_port, mem_index, &mem_list[mem_index], &backup_path);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 { print_usage(&args[0]); }

    let serial_port = &args[1];
    let command = &args[2];
    let data_0 = env::args().nth(3);
    let data_1 = env::args().nth(4);

    match &command as &str {
        "info" => print_info(serial_port),
        "freqrx" => cat::freq(serial_port, data_0, false),
        "freqtx" => cat::freq(serial_port, data_0, true),
        "backup" => backup(serial_port),
        "restore" => restore(serial_port, data_0, data_1),
        _ => print_usage(&args[0]),
    };
}
