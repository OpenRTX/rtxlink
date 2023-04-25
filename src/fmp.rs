//! This module handles the File Management Protocol portion of rtxlink

use std::fs::File;
use std::fs::metadata;
use std::thread;
use std::time::Duration;
use text_colorizer::*;
use thread_control::make_pair;
use ymodem::xmodem;

use crate::link::Errno;
use crate::link::Frame;
use crate::link::Link;
use crate::link::Protocol;

const OUTPUT_PATH: &str = "./flash_dump.bin";

/// FMP Protocol Opcodes
#[derive(PartialEq, Eq, Debug)]
pub enum Opcode {
    ACK     = 0x00,
    MEMINFO = 0x01,
    DUMP    = 0x02,
    FLASH   = 0x03,
    READ    = 0x04,
    WRITE   = 0x05,
    LIST    = 0x06,
    MOVE    = 0x07,
    COPY    = 0x08,
    MKDIR   = 0x09,
    RM      = 0x0a,
    RESET   = 0xff,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Opcode::ACK as u8 => Ok(Opcode::ACK),
            x if x == Opcode::MEMINFO as u8 => Ok(Opcode::MEMINFO),
            x if x == Opcode::DUMP as u8 => Ok(Opcode::DUMP),
            x if x == Opcode::FLASH as u8 => Ok(Opcode::FLASH),
            x if x == Opcode::READ as u8 => Ok(Opcode::READ),
            x if x == Opcode::WRITE as u8 => Ok(Opcode::WRITE),
            x if x == Opcode::LIST as u8 => Ok(Opcode::LIST),
            x if x == Opcode::MOVE as u8 => Ok(Opcode::MOVE),
            x if x == Opcode::COPY as u8 => Ok(Opcode::COPY),
            x if x == Opcode::MKDIR as u8 => Ok(Opcode::MKDIR),
            x if x == Opcode::RM as u8 => Ok(Opcode::RM),
            x if x == Opcode::RESET as u8 => Ok(Opcode::RESET),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct MemInfo {
    size: u32,
    name: [u8; 22],
    index: u8,
}

/// This function sends an FMP command
pub fn send_cmd(serial_port: &str, opcode: Opcode, params: Vec<Vec<u8>>) {
    let mut link = Link::new(serial_port);
    let mut cmd: Vec<u8> = vec![opcode as u8,
                                params.len() as u8];
    for p in params {
        cmd.push(p.len() as u8);
        cmd.extend(p);
    };
    let frame = Frame{proto: Protocol::FMP, data: cmd};
    link.send(frame);
}

/// This function reads the response of an FMP command, checking the error code
/// and returning the arguments of the response, possibly none
pub fn wait_reply(serial_port: &str, opcode: Opcode) -> Vec<Vec<u8>> {
    let mut link = Link::new(serial_port);
    // Loop until we get a message of the right protocol
    let mut frame: Frame;
    loop {
        frame = link.receive().expect("Error while reading frame");
        match frame.proto {
            Protocol::FMP => break,
            _ => (),
        };
    }
    let rx_opcode = Opcode::try_from(frame.data[0]).expect("Opcode not implemented!");
    if rx_opcode != opcode {
        eprintln!("{}: mismatched opcode in FMP response!", "Error".bold().red());
        return vec![]
    }
    // Parse status byte
    let status = frame.data[1];
    match status {
        0 => (),
        status => println!("{}: {:?}", "Error".bold().red(), Errno::try_from(status).unwrap()),
    }
    // Extract parameters
    vec![]
}

pub fn _xmodem_recv(_serial_port: &str, _output_file: &str) {
    // Create xmodem with 1K blocks
    let mut xmodem = xmodem::Xmodem::new();
    xmodem.block_length = xmodem::BlockLength::OneK;

    // Workaround for missing handle.is_running()
    // https://github.com/DenisKolodin/thread-control
    let (flag, control) = make_pair();
    let handle = thread::spawn(move || {
        if !flag.alive() { return; }
        println!("XMODEM transfer started");
        // TODO: Open serial port, open outfile
        //xmodem.recv(&mut serial_port, &mut output_file, xmodem::Checksum::CRC16)
        //      .expect("Failed to receive xmodem transfer");
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

/// Print info about the memories available on the platform
pub fn meminfo(serial_port: &str) -> String {
    send_cmd(serial_port, Opcode::MEMINFO, vec![]);
    // Receive MEMINFO response
    let data = wait_reply(serial_port, Opcode::MEMINFO);
    let (_, meminfo, _) = unsafe { data.align_to::<MemInfo>() };
    // Return MEMINFO response
    format!("{:?}", meminfo)
}

pub fn backup(_serial_port: &str) {
    // TODO: Enumerate all the memories, dump each in a file
    // meminfo(..)
    //xmodem_recv(..)
}

pub fn restore(serial_port: &str) {
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
    let (flag, _control) = make_pair();
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
