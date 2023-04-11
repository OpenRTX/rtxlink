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
| 0x03 | XMODEM command/frame |
```
*/

use crc::{Crc, Algorithm};
use serialport::SerialPort;
use slip::{encode, decode};
use std::convert::TryFrom;
use std::time::Duration;
use std::io::ErrorKind;

// rtxlink CRC8 polynomial
const CRC_ALGO: Algorithm<u8> = Algorithm {
    width: 8,
    poly: 0xa6,
    init: 0xff,
    refin: false,
    refout: false,
    xorout: 0x00,
    check: 0x6c,
    residue: 0x00
};

pub enum Protocol {
    STDIO = 0x00,
    CAT = 0x01,
    FMP = 0x02,
    XMODEM = 0x03
}

impl TryFrom<u8> for Protocol {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Protocol::STDIO as u8 => Ok(Protocol::STDIO),
            x if x == Protocol::CAT as u8 => Ok(Protocol::CAT),
            x if x == Protocol::FMP as u8 => Ok(Protocol::FMP),
            x if x == Protocol::XMODEM as u8 => Ok(Protocol::XMODEM),
            _ => Err(()),
        }
    }
}

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
        // Append the CRC8 using 0xA6 polynomial
        let crc = Crc::<u8>::new(&CRC_ALGO);
        let mut digest = crc.digest();
        digest.update(bin_rep.as_slice());
        bin_rep.push(digest.finalize());
        bin_rep
    }
}

pub struct Link {
    port: Box<dyn SerialPort>,
}

impl Link {
    pub fn new(port: String) -> Link {
        // This is the serial port used for the rtxlink connection
        Link{port: serialport::new(port, 115200).timeout(Duration::from_millis(10)).open().expect("Failed to open port")}
    }

    /// This function sends out a frame over a serial line, wrapped in slip
    /// and with the appropriate frame encoding.
    /// This function takes ownership of the Frame
    pub fn send(&mut self, frame: Frame) {
        // Generate binary representation of frame
        let bin_frame = frame.bin();
        let encoded: Vec<u8> = encode(&bin_frame).unwrap();
        println!("> {:x?}", &encoded);
        // Send frame down the serial port
        self.port.write(encoded.as_slice()).expect("Error in sending frame");
    }

    /// This function listens on the serial line for a frame, unwraps it,
    /// checks the CRC and returns it to the caller for dispatching.
    pub fn receive(&mut self) -> Result<Frame, ErrorKind> {
        let mut received: Vec<u8> = vec![0; 128];
        let nread = self.port.read(&mut received);
        println!("< {:x?}", &received);
        match nread {
            Ok(n) => received.resize(n, 0),
            Err(e) => panic!("Error while receiving data response: {e:?}")
        }

        // Validate and print response
        // TODO: support decoding multiple and incomplete packets
        let decoded: Vec<u8> = decode(&received).unwrap();
        // Check CRC
        let crc = Crc::<u8>::new(&CRC_ALGO);
        let mut digest = crc.digest();
        digest.update(decoded.as_slice());
        if digest.finalize() != 0x00 {
            return Err(ErrorKind::InvalidData);
        }
        // Assign correct protocol
        let proto = Protocol::try_from(decoded[0]).expect("Protocol not implemented!");
        Ok(Frame{proto: proto, data: decoded})
    }
}
