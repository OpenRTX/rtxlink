use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use ymodem::xmodem;

fn main() {

    let args: Vec<String> = env::args().collect();
    let tty_filename = args[1].clone();

    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    let mut tty_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(tty_filename)
        .expect("Failed to open serial device"); 
    let mut output_file = File::create("./flash_dump.bin").expect("Failed to open output file");

    let buffer = xmodem
                    .recv(&mut tty_file, &mut output_file, xmodem::Checksum::CRC16)
                    .expect("Failed to receive xmodem transfer"); 
}
