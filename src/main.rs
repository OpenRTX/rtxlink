use std::env;
use std::process;
use text_colorizer::*;
use std::sync::mpsc::channel;

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

fn cli_backup(port: String) {
    let (progress_tx, progress_rx) = channel();
    // Start backup thread
    std::thread::spawn(move || {
        rtxlink::link::Link::new(&port).expect("Error in opening serial port!");
        rtxlink::flow::backup(None, Some(&progress_tx));
    });
    // Progress printing loop
    let mut receive_size = 0;
    let mut size = 1;
    while receive_size < size {
        match progress_rx.recv() {
            Ok(x) => {
                (receive_size, size) = x;
                println!("Received: {receive_size:?}/{size:?}");
            }
            Err(_) => (),
        }
    }
}

fn cli_restore(port: String, mem_idx: Option<String>, file: Option<String>) {
    let (progress_tx, progress_rx) = channel();
    // Start backup thread
    std::thread::spawn(move || {
        rtxlink::link::Link::new(&port).expect("Error in opening serial port!");
        rtxlink::flow::restore(mem_idx, file, Some(&progress_tx));
    });
    // Progress printing loop
    let mut send_size = 0;
    let mut size = 1;
    while send_size < size {
        match progress_rx.recv() {
            Ok(x) => {
                (send_size, size) = x;
                println!("Sent: {send_size:?}/{size:?}");
            }
            Err(_) => (),
        }
    }
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
        "info" => { link::Link::new(serial_port).expect("Error in opening serial port!"); print_info() },
        "freqrx" => { link::Link::new(serial_port).expect("Error in opening serial port!"); cat::freq(data_0, false) },
        "freqtx" => { link::Link::new(serial_port).expect("Error in opening serial port!"); cat::freq(data_0, true) },
        "backup" => cli_backup(serial_port.clone()),
        "restore" => cli_restore(serial_port.clone(), data_0, data_1),
        _ => print_usage(&args[0]),
    };
}
