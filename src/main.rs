use std::env;
use std::process;

fn print_usage(cmd: &String) {
    println!("rtxlink: OpenRTX Communication Protocol");
    println!("usage: {cmd} SERIALPORT COMMAND [DATA]");
    println!("commands:");
    println!(" info                      Get device info");
    println!(" freqrx                    Print receive frequency");
    println!(" freqtx                    Print transmit frequency");
    println!(" freqrx FREQ_MHZ           Set the receive frequency");
    println!(" freqtx FREQ_MHZ           Set the transmit frequency");
    println!(" dump                      Read the device flash and save it to flash_dump.bin");
    println!(" flash                     Write an image to the device flash");
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print usage information
    if args.len() < 3 { print_usage(&args[0]); }

    let serial_port = args[1].clone();
    let command = args[2].clone();
    let data = env::args().nth(3);

    match &command as &str {
        "info" => rtxlink::info(serial_port),
        "freqrx" => rtxlink::freq(serial_port, data, false),
        "freqtx" => rtxlink::freq(serial_port, data, true),
        "dump" => rtxlink::dump(serial_port),
        "flash" => rtxlink::flash(serial_port),
        _ => print_usage(&args[0]),
    };
}
