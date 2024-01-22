//! This module handles the Computer Aided Transceiver portion of rtxlink

use byteorder::{ByteOrder, LittleEndian};
use std::str;

use crate::link::Errno;
use crate::link::Frame;
use crate::link::Link;
use crate::link::Protocol;

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
    INFO         = 0x494E, // IN
    FREQRX       = 0x5246, // RF
    FREQTX       = 0x5446, // TF
    FILETRANSFER = 0x4654, // FT
}

/// Convert Hertz in MegaHertz
const HZ_IN_MHZ: f64 = 1000000.0;

/// CAT GET request
fn get(id: ID) -> Vec<u8> {
    let mut link = Link::acquire();

    let cmd: Vec<u8> = vec![Opcode::GET as u8,
                            ((id as u16 >> 8) & 0xff) as u8,
                            (id as u16 & 0xff) as u8];
    let frame = Frame{proto: Protocol::CAT, data: cmd};
    link.send(frame);

    // Loop until we get a message of the right protocol
    let mut frame: Frame;
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
        Opcode::ACK => match data[1] {
            0 => (),
            status => println!("Error in GET request: {:?}", Errno::try_from(status).unwrap()),
        }, // Error?
        Opcode::DATA => { data.remove(0); () }, // Correct response!
        _ => panic!("Error while parsing GET response"),
    };
    link.release();
    data
}

/// CAT SET request
fn set(id: ID, data: &[u8]) {
    let mut link = Link::acquire();

    let mut cmd: Vec<u8> = vec![Opcode::SET as u8,
                                ((id as u16 >> 8) & 0xff) as u8,
                                (id as u16 & 0xff) as u8];
    cmd.extend(data);
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
    let data = frame.data;
    let opcode = Opcode::try_from(data[0]).expect("Opcode not implemented!");
    match opcode {
        Opcode::ACK => match data[1] {
            0 => (),
            status => println!("Error in SET request: {:?}", Errno::try_from(status).unwrap()),
        }, // Error?
        _ => panic!("Error while parsing SET response"),
    };
    link.release();
}

/// CAT GET radio info
pub fn info() -> String {
    let data: Vec<u8> = get(ID::INFO);
    match str::from_utf8(&data) {
        Ok(name) => String::from(name),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

/// CAT GET or SET radio frequency
pub fn freq(data: Option<String>, is_tx: bool) {
    let id = if is_tx { ID::FREQTX } else { ID::FREQRX };
    // If user supplied no data print frequency, otherwise set
    match data {
        // GET
        None => {
            let data: Vec<u8> = get(id);
            let freq: u32 = LittleEndian::read_u32(&data);
            let freq: f64 = freq as f64 / HZ_IN_MHZ;
            match is_tx {
                true => println!("Tx: {freq} MHz"),
                false => println!("Rx: {freq} MHz"),
            };
        },
        // SET
        Some(data) => {
            let freq: f64 = data.parse::<f64>().unwrap();
            let freq: u32 = (freq * HZ_IN_MHZ) as u32;
            let mut data: [u8; 4] = [0, 0, 0, 0];
            LittleEndian::write_u32(&mut data, freq);
            set(id, &data);
        },
    };
}

/// CAT SET file transfer mode
pub fn ftm() {
    let data: [u8; 0] = [];
    set(ID::FILETRANSFER, &data);
}
