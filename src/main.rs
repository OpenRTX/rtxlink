use std::env;
use std::process;

fn main() {

    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 {
        println!("rtxlink: OpenRTX Communication Protocol");
        println!("usage: {} COMMAND SERIALPORT", args[0]);
        println!("commands:");
        println!(" info                       Get device info");
        println!(" dump                       Read the device flash and save it to flash_dump.bin");
        println!(" flash                      Write an image to the device flash");
        process::exit(0);
    }

    let command = args[1].clone();
    let serial_port = args[2].clone();

    if command == "info" { rtxlink::info(serial_port); }
    else if command == "dump" { rtxlink::dump(serial_port); }
    else if command == "flash" { rtxlink::flash(serial_port); }
}
