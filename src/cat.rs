//! This module handles the Computer Aided Transceiver portion of rtxlink

use byteorder::{ByteOrder, LittleEndian};
use std::str;

use crate::link::Link;
use crate::link::Protocol;
use crate::link::Frame;

// CAT Protocol opcodes
enum Opcode {
    GET = 0x47, // G
    SET = 0x53, // S
    DATA = 0x44, // D
    _ACK = 0x41, // A
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Opcode::GET as u8 => Ok(Opcode::GET),
            x if x == Opcode::SET as u8 => Ok(Opcode::SET),
            x if x == Opcode::DATA as u8 => Ok(Opcode::DATA),
            x if x == Opcode::_ACK as u8 => Ok(Opcode::_ACK),
            _ => Err(()),
        }
    }
}

// CAT Protocol IDs
const ID_INFO:   (u8, u8) = (0x49, 0x4E); // IN
const ID_FREQRX: (u8, u8) = (0x52, 0x46); // FR
const ID_FREQTX: (u8, u8) = (0x54, 0x46); // FT

// Convert Hertz in MegaHertz
const HZ_IN_MHZ: f32 = 1000000.0;

fn get(serial_port: String, id: (u8, u8)) -> Vec<u8> {
    let mut link = Link::new(serial_port);

    let cmd: Vec<u8> = vec![Opcode::GET as u8, id.0, id.1];
    let frame = Frame{proto: Protocol::CAT, data: cmd};
    link.send(frame);

    let mut frame: Frame;
    // Loop until we get a message of the right protocol
    loop {
        frame = link.receive().expect("Error while reading frame");
        match frame.proto {
            Protocol::CAT => break,
            _ => (),
        };
    }
    let mut data = frame.data;
    let opcode = Opcode::try_from(data[0]).expect("Opcode not implemented!");
    match opcode {
        Opcode::DATA => data.remove(0),
        _ => panic!("Error while parsing info response"),
    };
    data
}

fn set(serial_port: String, id: (u8, u8), data: &[u8]) {
    let mut link = Link::new(serial_port);

    let mut cmd: Vec<u8> = vec![Opcode::SET as u8, id.0, id.1];
    cmd.extend(data);
    let frame = Frame{proto: Protocol::CAT, data: cmd};
    link.send(frame);

    let frame = link.receive().expect("Error in frame reception");
    let data = match frame.proto {
        Protocol::CAT => frame.data,
        _ => panic!("Error: wrong protocol received"),
    };
    println!("{:?}", data);
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
