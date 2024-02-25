use std::env;
use std::process;
use text_colorizer::*;

mod cat;
mod dat;
mod flow;
mod fmp;
mod link;
mod slip;

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
fn print_info() {
    println!("Radio identifier: {}", cat::info());
    let mem_list = fmp::meminfo();
    println!("Available memories:");
    let mut i: usize = 0;
    for mem in mem_list {
        println!("[{}]: {:?}", i, mem);
        i += 1;
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 { print_usage(&args[0]); }

    let serial_port = &args[1];
    let command = &args[2];
    let data_0 = env::args().nth(3);
    let data_1 = env::args().nth(4);

    // Open serial port
    link::Link::new(serial_port);

    match &command as &str {
        "info" => print_info(),
        "freqrx" => cat::freq(data_0, false),
        "freqtx" => cat::freq(data_0, true),
        "backup" => flow::backup(None),
        "restore" => flow::restore(data_0, data_1),
        _ => print_usage(&args[0]),
    };
}
