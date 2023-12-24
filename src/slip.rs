use std::io::{Error, ErrorKind, Result};

pub const END: u8 = 0xC0;
pub const ESC: u8 = 0xDB;
pub const ESC_END: u8 = 0xDC;
pub const ESC_ESC: u8 = 0xDD;

pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut encoded_data = Vec::new();
    encoded_data.push(END);

    for byte in data {
        match byte {
            &END => {
                encoded_data.push(ESC);
                encoded_data.push(ESC_END);
            }
            &ESC => {
                encoded_data.push(ESC);
                encoded_data.push(ESC_ESC);
            }
            _ => encoded_data.push(*byte),
        }

    }

    encoded_data.push(END);
    encoded_data
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoded_data = Vec::new();
    let mut escaped = false;

    for byte in data {
        match byte {
            &END => break,
            &ESC => {
                escaped = true;
            }
            &ESC_END if escaped => {
                decoded_data.push(END);
                escaped = false;
            }
            &ESC_ESC if escaped => {
                decoded_data.push(ESC);
                escaped = false;
                ()
            }
            _ => {
                if escaped {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid SLIP escape sequence"));
                }
                decoded_data.push(*byte);
                ()
            }
        }
    }
    
    Ok(decoded_data)
}

pub fn decode_frames(data: &[u8]) -> Result<(Vec<Vec<u8>>, &[u8])> {
    let mut frames = Vec::new();
    let mut packet = Vec::new();
    let mut escaped = false;
    let mut in_packet = false;
    let mut remainder_index = 0;

    for i in 0..data.len() {
        match data[i] {
            END => {
                // Discard all bytes until the first END
                if !in_packet {
                    in_packet = true;
                    remainder_index = i;
                } else {
                    // Completed one SLIP frame
                    if !packet.is_empty() {
                        frames.push(packet);
                        packet = Vec::new();
                        in_packet = false;
                        remainder_index = i + 1;
                    };
                }
            }
            ESC if in_packet => {
                escaped = true;
            }
            ESC_END if in_packet && escaped => {
                packet.push(END);
                escaped = false;
            }
            ESC_ESC if in_packet && escaped => {
                packet.push(ESC);
                escaped = false;
            }
            _ => {
                if escaped {
                    return Err(Error::new(ErrorKind::InvalidData, "Invalid SLIP escape sequence"));
                }
                if in_packet {
                    packet.push(data[i]);
                } 
            }
        }
    }
    
    // Return unused bytes
    Ok((frames, &data[remainder_index..]))
}
