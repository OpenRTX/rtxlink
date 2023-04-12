//! This module handles the Computer Aided Transceiver portion of rtxlink

use byteorder::{ByteOrder, LittleEndian};
use std::str;

use crate::link::Link;
use crate::link::Protocol;
use crate::link::Frame;

/// CAT Protocol opcodes
enum Opcode {
    GET  = 0x47, // G
    SET  = 0x53, // S
    DATA = 0x44, // D
    ACK  = 0x41, // A
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Opcode::GET as u8 => Ok(Opcode::GET),
            x if x == Opcode::SET as u8 => Ok(Opcode::SET),
            x if x == Opcode::DATA as u8 => Ok(Opcode::DATA),
            x if x == Opcode::ACK as u8 => Ok(Opcode::ACK),
            _ => Err(()),
        }
    }
}

/// CAT Protocol IDs
#[derive(Copy, Clone)]
enum ID {
    INFO   = 0x494E, // IN
    FREQRX = 0x5246, // FR
    FREQTX = 0x5446, // FT

}

/// POSIX Errors
#[derive(Debug)]
enum Errno {
    OK      = 0,    // Success
    E2BIG   = 7,    // Argument list too long
    EBADR   = 53,   // Invalid request descriptor
    EBADRQC = 56,   // Invalid request code
    EGENERIC = 255, // Generic error
}

impl TryFrom<u8> for Errno {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Errno::OK as u8 => Ok(Errno::OK),
            x if x == Errno::E2BIG as u8 => Ok(Errno::E2BIG),
            x if x == Errno::EBADR as u8 => Ok(Errno::EBADR),
            x if x == Errno::EBADRQC as u8 => Ok(Errno::EBADRQC),
            x if x == Errno::EGENERIC as u8 => Ok(Errno::EGENERIC),
            _ => Err(()),
        }
    }
}

/// Convert Hertz in MegaHertz
const HZ_IN_MHZ: f32 = 1000000.0;

/// CAT GET request
fn get(serial_port: String, id: ID) -> Vec<u8> {
    let mut link = Link::new(serial_port);

    let cmd: Vec<u8> = vec![Opcode::GET as u8,
                            ((id as u16 >> 8) & 0xff) as u8,
                            (id as u16 & 0xff) as u8];
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
    println!("{:x?}", frame.data);
    let mut data = frame.data;
    let opcode = Opcode::try_from(data[0]).expect("Opcode not implemented!");
    match opcode {
        Opcode::ACK => match data[1] {
            0 => (),
            status => println!("Error in GET request: {:?}", Errno::try_from(status).unwrap()),
        }, // Error?
        Opcode::DATA => { data.remove(0); () }, // Correct response!
        _ => panic!("Error while parsing info response"),
    };
    data
}

/// CAT SET request
fn set(serial_port: String, id: ID, data: &[u8]) {
    let mut link = Link::new(serial_port);

    let mut cmd: Vec<u8> = vec![Opcode::SET as u8,
                                ((id as u16 >> 8) & 0xff) as u8,
                                (id as u16 & 0xff) as u8];
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

/// CAT GET radio info
pub fn info(serial_port: String) {
    let data: Vec<u8> = get(serial_port, ID::INFO);
    match str::from_utf8(&data) {
        Ok(name) => println!("OpenRTX: {name}"),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
}

/// CAT GET or SET radio frequency
pub fn freq(serial_port: String, data: Option<String>, is_tx: bool) {
    let id = if is_tx { ID::FREQTX } else { ID::FREQRX };
    // If user supplied no data print frequency, otherwise set
    match data {
        // GET
        None => {
            let data: Vec<u8> = get(serial_port, id);
            let freq: u32 = LittleEndian::read_u32(&data);
            let freq: f32 = freq as f32 / HZ_IN_MHZ;
            match is_tx {
                true => println!("Tx: {freq} MHz"),
                false => println!("Rx: {freq} MHz"),
            };
        },
        // SET
        Some(data) => {
            let freq: f32 = data.parse::<f32>().unwrap();
            let freq: u32 = (freq * HZ_IN_MHZ) as u32;
            let mut data: [u8; 4] = [0, 0, 0, 0];
            LittleEndian::write_u32(&mut data, freq);
            set(serial_port, id, &data);
        },
    };
}
