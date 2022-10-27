use std::fs::{File, metadata};
use std::thread;
use std::time::Duration;
use std::str;
use slip::{encode, decode};
use thread_control::*;
use ymodem::xmodem;

// CAT Protocol opcodes
const GET:  u8 = 0x47; // G
const SET:  u8 = 0x53; // S
const DATA: u8 = 0x44; // D
const ACK:  u8 = 0x41; // A

// CAT Protocol IDs
const GET_INFO_H: u8 = 0x49;
const GET_INFO_L: u8 = 0x4E;

const OUTPUT_PATH: &str = "./flash_dump.bin";

pub fn info(serial_port: String) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");

    let cmd: Vec<u8> = vec![GET, GET_INFO_H, GET_INFO_L];
    let encoded: Vec<u8> = encode(&cmd).unwrap();

    port.write(&encoded);
    let mut received: Vec<u8> = vec![0; 128];
    let nread = port.read(&mut received);
    match nread {
        Ok(n) => {
            received.resize(n, 0);
        }
        Err(e) => {
            eprintln!("Error while receiving info response: {e:?}");
            std::process::exit(-1);
        }
    }

    // Validate and print response
    let decoded: Vec<u8> = decode(&received).unwrap();
    match decoded[0] {
        DATA => {
            match str::from_utf8(&decoded[2..]) {
                Ok(name) => println!("OpenRTX: {name}"),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
        },
        _ => {
            eprintln!("Error while parsing info response");
            std::process::exit(-1);
        }
    }
}

pub fn dump(serial_port: String) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");
    let mut output_file = File::create(OUTPUT_PATH)
        .expect("Failed to open output file");

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Workaround for missing handle.is_running()
    // https://github.com/DenisKolodin/thread-control
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        if !flag.alive() { return; }
        println!("XMODEM transfer started");
        xmodem.recv(&mut port, &mut output_file, xmodem::Checksum::CRC16)
              .expect("Failed to receive xmodem transfer");
        println!("XMODEM transfer finished");
    });

    // handle.is_running() is not available yet, use it when it is released
    // https://github.com/rust-lang/rust/issues/90470
    while !control.is_done() {
        let output_size = metadata(OUTPUT_PATH).unwrap().len();
        println!("{} size: {} Bytes", OUTPUT_PATH, output_size);
        thread::sleep(Duration::from_millis(1000));
    }
    // Wait for xmodem thread to finish
    handle.join().unwrap();
}

pub fn flash(serial_port: String) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_secs(60))
        .open().expect("Failed to open serial port");
    let mut output_file = File::open(OUTPUT_PATH)
        .expect("Failed to open input file");

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Workaround for missing handle.is_running()
    // https://github.com/DenisKolodin/thread-control
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        if !flag.alive() { return; }
        println!("XMODEM transfer started");
        xmodem.send(&mut port, &mut output_file)
              .expect("Failed to send xmodem transfer");
        println!("XMODEM transfer finished");
    });

    thread::sleep(Duration::from_millis(500));
    println!("Press PTT on the radio to start XMODEM transfer");
    // Wait for xmodem thread to finish
    handle.join().unwrap();
}
