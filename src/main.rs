use std::env;
use std::process;

fn print_usage(cmd: &String) {
    println!("rtxlink: OpenRTX Communication Protocol");
    println!("usage: {cmd} COMMAND SERIALPORT");
    println!("commands:");
    println!(" info                       Get device info");
    println!(" freqrx                     Print receive frequency");
    println!(" freqtx                     Print transmit frequency");
    println!(" dump                       Read the device flash and save it to flash_dump.bin");
    println!(" flash                      Write an image to the device flash");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 { print_usage(&args[0]); }

    let command = args[1].clone();
    let serial_port = args[2].clone();

    if command == "info" { rtxlink::info(serial_port); }
    else if command == "freqrx" { rtxlink::freqrx(serial_port); }
    else if command == "freqtx" { rtxlink::freqtx(serial_port); }
    else if command == "dump" { rtxlink::dump(serial_port); }
    else if command == "flash" { rtxlink::flash(serial_port); }
    else { print_usage(&args[0]); }
}
