//! This module handles the File Management Protocol portion of rtxlink

use std::ffi::CStr;
use std::fmt;
use std::str;
use text_colorizer::*;
use std::sync::mpsc::Sender;

use crate::link::Errno;
use crate::link::Frame;
use crate::link::Link;
use crate::link::Protocol;
use crate::dat;

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
    size:  u32,     // Size of the memory in Bytes
    flags: u8,      // Flags
    name: [u8; 27], // Name of the memory
}

// Useful for terminal printing
impl fmt::Debug for MemInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{} ({}B)",
               str::from_utf8(&self.name).unwrap(),
               self.size)
    }
}

// Used for deriving file names
impl std::fmt::Display for MemInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}_{}",
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
pub fn send_cmd(opcode: Opcode, params: Vec<Vec<u8>>) {
    let mut link = Link::acquire();
    let mut cmd: Vec<u8> = vec![opcode as u8,
                                params.len() as u8];
    for p in params {
        cmd.push(p.len() as u8);
        cmd.extend(p);
    };
    let frame = Frame{proto: Protocol::FMP, data: cmd};
    link.send(frame);
    link.release();
}

/// This function reads the response of an FMP command, checking the error code
/// and returning the arguments of the response, possibly none
pub fn wait_reply(opcode: Opcode) -> Vec<Vec<u8>> {
    let mut link = Link::acquire();
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
    link.release();
    params
}

/// Print info about the memories available on the platform
pub fn meminfo() -> Vec<MemInfo> {
    send_cmd(Opcode::MEMINFO, vec![]);
    // Receive MEMINFO response
    let available_mem = wait_reply(Opcode::MEMINFO);
    // Return MEMINFO response
    let mem_list = available_mem.iter()
                                .map(|m| MemInfo::from(m))
                                .collect();
    mem_list
}

/// Dump memory device into a file
pub fn dump(mem_id: usize, mem: &MemInfo, file_name: &str, progress: Option<&Sender<(usize, usize)>>) -> std::io::Result<()> {
    // Send Dump FMP command then listen for incoming DAT transfer
    send_cmd(Opcode::DUMP, [[mem_id as u8].to_vec()].to_vec());
    wait_reply(Opcode::DUMP);
    dat::receive(file_name, mem.size as usize, progress)
}

/// Flash a given file into a particular memory device of a radio
pub fn flash(mem_id: usize, mem: &MemInfo, file_name: &str, progress: Option<&Sender<(usize, usize)>>) {
    // Send Fump FMP command then send content over DAT
    send_cmd(Opcode::FLASH, [[mem_id as u8].to_vec()].to_vec());
    wait_reply(Opcode::FLASH);
    dat::send(file_name, mem.size as usize, progress);
}
