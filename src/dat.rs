//! This module handles the Data Transfer Protocol portion of rtxlink

use text_colorizer::*;
use std::fs::{File, read};
use std::io::Write;
use std::io::{Error, ErrorKind};

use crate::link::Errno;
use crate::link::Frame;
use crate::link::Link;
use crate::link::Protocol;

const DAT_FRAME_SIZE: usize = 1024;
const DAT_PAYLOAD_SIZE: usize = DAT_FRAME_SIZE - 2;

/// This function sends an ACK to signal the correct reception of a DAT frame
pub fn send_ack(link: &mut Link) {
    let frame = Frame{proto: Protocol::DAT, data: vec![0x06]};
    link.send(frame);
}

/// This function sends an ACK to signal the correct reception of a DAT frame
pub fn wait_ack() {
    let mut link = Link::acquire();
    // Loop until we get a message of the right protocol
    let mut frame: Frame;
    loop {
        frame = link.receive().expect("Error while reading frame");
        match frame.proto {
            Protocol::DAT => break,
            _ => (),
        };
    }
    // Parse status byte
    let ack = frame.data[0];
    match ack {
        0x06 => (),
        status => println!("{}: {:?}", "Error".bold().red(), Errno::try_from(status).unwrap()),
    }
    link.release();
}

/// This function receives data using the DAT protocol
pub fn receive(file_name: &str, size: usize) -> std::io::Result<()> {
    let mut receive_size: usize = 0;
    let mut prev_block: i16 = -1;
    let mut file = File::create(&file_name)?;
    let mut link = Link::acquire();
    // Loop until we get a message of the right protocol
    let mut frame: Frame;
    send_ack(&mut link);
    while receive_size != size {
        loop {
            frame = link.receive().expect("Error while reading frame");
            match frame.proto {
                Protocol::DAT => break,
                _ => (),
            };
        }
        // Check sanity of block number and its inverse
        let block_number = frame.data[0];
        let inv_block_number = frame.data[1];
        if (block_number + inv_block_number != 255) ||
           (block_number != (prev_block + 1) as u8) {
            return Err(Error::new(ErrorKind::Other, "Error in DAT protocol receive: bad block indexing!"));
        }
        prev_block = block_number as i16;
        receive_size += frame.data.len() - 2;
        file.write_all(&frame.data[2..])?;
        send_ack(&mut link);
    }
    link.release();
    Ok(())
}

/// This function sends data using the DAT protocol
pub fn send(file_name: &str, size: usize) {
    let file_content = read(&file_name).expect("Error in reading backup file!");
    if size != file_content.len() {
        panic!("Backup file does not match with memory size!");
    }
    // Send chunks of 1022B
    for i in 1..(size / DAT_PAYLOAD_SIZE) + 2 {
        // Set frame counter and reverse frame counter
        let mut chunk: Vec<u8> = vec![0; DAT_FRAME_SIZE];
        chunk[0] = i as u8;
        chunk[1] = 255 - i as u8;
        let remaining_data: usize = size - (i - 1) * DAT_PAYLOAD_SIZE;
        let chunk_size: usize = if remaining_data < DAT_PAYLOAD_SIZE {remaining_data} else {DAT_PAYLOAD_SIZE};
        let start_offset = (i-1) * DAT_PAYLOAD_SIZE;
        let end_offset = start_offset + chunk_size;
        chunk[2..chunk_size + 2].copy_from_slice(&file_content[start_offset..end_offset]);
        chunk.resize(chunk_size, 0);
        let mut link = Link::acquire();
        let frame = Frame{proto: Protocol::DAT, data: chunk};
        link.send(frame);
        link.release();
        wait_ack();
    }
}
