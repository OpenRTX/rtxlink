use std::fs::{File, metadata};
use std::str;
use std::thread;
use std::time::Duration;
use byteorder::{ByteOrder, LittleEndian};
use slip::{encode, decode};
use thread_control::*;
use ymodem::xmodem;

// CAT Protocol opcodes
const GET:  u8 = 0x47; // G
const SET:  u8 = 0x53; // S
const DATA: u8 = 0x44; // D
const ACK:  u8 = 0x41; // A

// CAT Protocol IDs
const ID_INFO:   (u8, u8) = (0x49, 0x4E); // IN
const ID_FREQRX: (u8, u8) = (0x52, 0x46); // FR
const ID_FREQTX: (u8, u8) = (0x54, 0x46); // FT

// Convert Hertz in MegaHertz
const HZ_IN_MHZ: f32 = 1000000.0;

const OUTPUT_PATH: &str = "./flash_dump.bin";

fn get(serial_port: String, id: (u8, u8)) -> Vec<u8> {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");

    let cmd: Vec<u8> = vec![GET, id.0, id.1];
    let encoded: Vec<u8> = encode(&cmd).unwrap();

    match port.write(&encoded) {
        Err(e) => panic!("Error while sending get request: {e:?}"),
        Ok(v) => v
    };
    let mut received: Vec<u8> = vec![0; 128];
    let nread = port.read(&mut received);
    match nread {
        Ok(n) => received.resize(n, 0),
        Err(e) => panic!("Error while receiving data response: {e:?}")
    };

    // Validate and print response
    let decoded: Vec<u8> = decode(&received).unwrap();
    //println!("{:?}", received);
    match decoded[0] {
        DATA => return decoded,
        _ => panic!("Error while parsing info response"),
    }
}

fn set(serial_port: String, id: (u8, u8), data: &[u8]) {
    let mut port = serialport::new(serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");

    let mut cmd: Vec<u8> = vec![SET, id.0, id.1];
    cmd.push(data.len() as u8);
    cmd.extend(data);
    let encoded: Vec<u8> = encode(&cmd).unwrap();
    println!("{:?}", encoded);

    match port.write(&encoded) {
        Err(e) => panic!("Error while sending set request: {e:?}"),
        Ok(v) => v
    };
    let mut received: Vec<u8> = vec![0; 128];
    let nread = port.read(&mut received);
    match nread {
        Ok(n) => received.resize(n, 0),
        Err(e) => panic!("Error while receiving ACK: {e:?}")
    };

    // Validate and print response
    let decoded: Vec<u8> = decode(&received).unwrap();
    println!("{:?}", received);
    // TODO: Validate ACK
}

pub fn info(serial_port: String) {
    let data: Vec<u8> = get(serial_port, ID_INFO);
    match str::from_utf8(&data[2..]) {
        Ok(name) => println!("OpenRTX: {name}"),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
}

pub fn freq(serial_port: String, data: Option<String>, is_tx: bool) {
    let id: (u8, u8) = if is_tx { ID_FREQTX } else { ID_FREQRX };
    // If user supplied no data print frequency, otherwise set
    match data {
        None => {
            let data: Vec<u8> = get(serial_port, id);
            let freq: u32 = LittleEndian::read_u32(&data[2..]);
            let freq: f32 = freq as f32 / HZ_IN_MHZ;
            match is_tx {
                true => println!("Tx: {freq} MHz"),
                false => println!("Rx: {freq} MHz"),
            };
        },
        Some(data) => {
            let freq: f32 = data.parse::<f32>().unwrap();
            let freq: u32 = (freq * HZ_IN_MHZ) as u32;
            let mut data: [u8; 4] = [0, 0, 0, 0];
            LittleEndian::write_u32(&mut data, freq);
            set(serial_port, id, &data);
        },
    };
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
