use std::env;
use std::fs::File;
use std::time::Duration;
use ymodem::xmodem;

fn main() {

    let args: Vec<String> = env::args().collect();
    let tty_dev = args[1].clone();

    let mut port = serialport::new(tty_dev, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open serial port");
    let mut output_file = File::create("./flash_dump.bin")
        .expect("Failed to open output file");

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Receive full flash copy
    xmodem.recv(&mut port, &mut output_file, xmodem::Checksum::CRC16)
          .expect("Failed to receive xmodem transfer"); 
}
