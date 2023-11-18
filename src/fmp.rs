//! This module handles the File Management Protocol portion of rtxlink

use chrono;
use std::ffi::CStr;
use std::fmt;
use std::fs::File;
use std::fs::metadata;
use std::fs::OpenOptions;
use std::str;
use std::thread;
use std::time::Duration;
use text_colorizer::*;
use thread_control::make_pair;

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemInfo {
    size: u32,      // Size of the memory in Bytes
    name: [u8; 24], // Name of the memory
    index: u32,     // Index for referencing this memory with FMP commands
}

// Useful for terminal printing
impl fmt::Debug for MemInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Index: {}, Name: {}, Size: {}",
               self.index,
               str::from_utf8(&self.name).unwrap(),
               self.size)
    }
}

// Used for deriving file names
impl std::fmt::Display for MemInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}_{}_{}",
               self.index,
               unsafe{ CStr::from_ptr(self.name.as_ptr() as *const i8).to_str()
                                                                      .unwrap()
                                                                      .replace(" ", "") },
               self.size)
    }
}

impl From<&Vec<u8>> for MemInfo {
    fn from(v: &Vec<u8>) -> MemInfo {
        unsafe{ *std::mem::transmute::<*const u8, *const MemInfo>(v.as_ptr()) }
    }
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
    let nparams = frame.data[2] as usize;
    let mut params = Vec::new();
    let mut prev_params: usize = 0;
    for _i in 0..nparams {
        // Keep track of the offset
        let param_size: usize = frame.data[3 + _i] as usize;
        let mut param = Vec::with_capacity(param_size);
        // Skip FMP header, param sizes and previous params
        for j in 3 + nparams + prev_params..3 + nparams + prev_params + param_size {
            param.push(frame.data[j]);
        }
        params.push(param);
        prev_params += param_size;
    }
    params
}

/// Print info about the memories available on the platform
pub fn meminfo(serial_port: &str) -> Vec<MemInfo> {
    send_cmd(serial_port, Opcode::MEMINFO, vec![]);
    // Receive MEMINFO response
    let available_mem = wait_reply(serial_port, Opcode::MEMINFO);
    // Return MEMINFO response
    let mem_list = available_mem.iter()
                                .map(|m| MemInfo::from(m))
                                .collect();
    mem_list
}

pub fn backup(serial_port: &str) {
    // Enumerate all the memories, dump each in a separate file
    let mem_list = meminfo(serial_port);
    for mem in mem_list {
        let mut file_name: String = String::from("");
        file_name.push_str(&chrono::offset::Local::now().format("%d%m%Y_")
                                                        .to_string());
        file_name.push_str(&mem.to_string());
        file_name.push_str(".bin");
        dump(serial_port, mem.index, &file_name);
    }
}

// TODO:
pub fn dump(serial_port: &str, mem_id: u32, file_name: &str);

// TODO:
pub fn restore(serial_port: &str);
