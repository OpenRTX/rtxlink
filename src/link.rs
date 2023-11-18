//! This module handles the Data Link Layer of the rtxlink communication protocol

/*!
## Frame Format

```
|  0  |    1    |  ... |  N-1 |  N  |
|:---:|:-------:|:----:|:----:|:---:|
| END | ProtoID | Data | CRC8 | END |
```

Following the leading END marker, the first byte of each frame is a protocol identifier describing the frame content, while the last byte of the frame contains the CRC-8 of the protocol ID and data fields. The polynomial used for the CRC is 0xA6, ensuring a minimum hamming distance of 2 for data blocks composed by more than 2048 bytes. 

The recognized protocol IDs are the following:

```
|  ID  |    Frame content     |
|:----:|:--------------------:|
| 0x00 | stdio redirection    |
| 0x01 | CAT command/response |
| 0x02 | FMP command/response |
| 0x03 | DAT frame/ack        |
```
*/

use crc16::*;
use serialport::SerialPort;
use slip::{encode, decode_packets};
use std::convert::TryFrom;
use std::time::Duration;
use std::io::ErrorKind;

#[derive(Debug)]
pub enum Protocol {
    STDIO = 0x00,
    CAT = 0x01,
    FMP = 0x02,
    DAT = 0x03
}

/// POSIX Errors
#[derive(Debug)]
pub enum Errno {
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

impl TryFrom<u8> for Protocol {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Protocol::STDIO as u8 => Ok(Protocol::STDIO),
            x if x == Protocol::CAT as u8 => Ok(Protocol::CAT),
            x if x == Protocol::FMP as u8 => Ok(Protocol::FMP),
            x if x == Protocol::DAT as u8 => Ok(Protocol::DAT),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub proto: Protocol,
    pub data: Vec<u8>,
}

impl Frame {
    /// This function generates the binary representation of a Frame.
    /// This function takes ownership of the Frame
    fn bin(self) -> Vec<u8> {
        // Copy from data to array
        let mut bin_rep = Vec::from(self.data);
        // Prepend the Protocol IDentifier
        bin_rep.insert(0, self.proto as u8);
        // Append the CRC16 using CCITT polynomial
        let digest = State::<AUG_CCITT>::calculate(bin_rep.as_slice());
        bin_rep.push((digest & 0xff) as u8);
        bin_rep.push((digest >> 8 & 0xff) as u8);
        bin_rep
    }
}

pub struct Link {
    port: Box<dyn SerialPort>,
}

impl Link {
    pub fn new(port: &str) -> Link {
        // This is the serial port used for the rtxlink connection
        Link{port: serialport::new(port, 115200).timeout(Duration::from_millis(10))
                                                .open()
                                                .expect("Failed to open port")}
    }

    /// This function sends out a frame over a serial line, wrapped in slip
    /// and with the appropriate frame encoding.
    /// This function takes ownership of the Frame
    pub fn send(&mut self, frame: Frame) {
        // Generate binary representation of frame
        let bin_frame = frame.bin();
        let encoded: Vec<u8> = encode(&bin_frame).unwrap();
        // Send frame down the serial port
        self.port.write(encoded.as_slice()).expect("Error in sending frame");
    }

    /// This function listens on the serial line for a frame, unwraps it,
    /// checks the CRC and returns it to the caller for dispatching.
    pub fn receive(&mut self) -> Result<Frame, ErrorKind> {
        // Enqueue data until we get the first valid packet
        let mut remainder: Vec<u8> = vec![];
        let frames: Vec<Vec<u8>> = loop {
            let mut received: Vec<u8> = vec![0; 128];
            let nread = self.port.read(&mut received);
            //println!("Rx: {:?}", received);
            match nread {
                Ok(n) => received.resize(n, 0),
                Err(e) => panic!("Error while receiving data response: {e:?}")
            };
            remainder.extend(received);
            remainder.shrink_to_fit();

            // Decode SLIP framing
            let (frames, _remainder) = decode_packets(&remainder);
            if frames.len() > 0 {
                break frames
            }
        };

        // Check CRC16 using CCITT polynomial
        let digest = State::<AUG_CCITT>::calculate(&frames[0]);
        if digest != 0x0000 {
            return Err(ErrorKind::InvalidData);
        }
        // Assign correct protocol
        let proto = Protocol::try_from(frames[0][0]).expect("Protocol not implemented!");
        // Trim proto (1 byte at beginning) and CRC (1 byte at end)
        let data = &frames[0][1..frames[0].len() - 2];
        let frame = Frame {proto: proto, data: Vec::from(data)};
        Ok(frame)
    }
}
