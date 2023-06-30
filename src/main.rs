use std::env;
use std::process;
use text_colorizer::*;

mod link;
mod cat;
mod fmp;

/// Print usage information of this tool
fn print_usage(cmd: &String) {
    eprintln!("{}: OpenRTX Communication Protocol", "rtxlink".yellow());
    eprintln!("{}: invalid parameters", "Error".red().bold());
    eprintln!("Usage: {cmd} SERIALPORT COMMAND [DATA]");
    eprintln!("commands:");
    eprintln!(" info                      Get device info");
    eprintln!(" freqrx                    Print receive frequency");
    eprintln!(" freqtx                    Print transmit frequency");
    eprintln!(" freqrx FREQ_MHZ           Set the receive frequency");
    eprintln!(" freqtx FREQ_MHZ           Set the transmit frequency");
    eprintln!(" dump                      Read the device flash and save it to flash_dump.bin");
    eprintln!(" flash                     Write an image to the device flash");
    process::exit(1);
}

/// Print info about the target OpenRTX platform
fn print_info(serial_port: &str) {
    println!("Radio identifier: {}", cat::info(serial_port));
    let memlist = fmp::meminfo(serial_port);
    println!("Available memories:");
    for mem in memlist {
        println!("- {:?}", mem);
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 { print_usage(&args[0]); }

    let serial_port = &args[1];
    let command = &args[2];
    let data = env::args().nth(3);

    match &command as &str {
        "info" => print_info(serial_port),
        "freqrx" => cat::freq(serial_port, data, false),
        "freqtx" => cat::freq(serial_port, data, true),
        "backup" => fmp::backup(serial_port),
        "restore" => fmp::restore(serial_port),
        _ => print_usage(&args[0]),
    };
}
