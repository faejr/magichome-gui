use std::net::{TcpStream};
use std::io::{Read, Write};

use super::{Mode, Output, State};

const CUSTOM_PATTERN_MODE: u8 = 0x51;
const LIST_TERMINATOR: u8 = 0x00;
const JUMP_MODE: u8 = 0x3B;
const PATTERN_TERMINATOR_1: u8 = 0xFF;
const PATTERN_TERMINATOR_2: u8 = 0x0F;
const PATTERN_SPEED: u8 = 0x1; // max 31 (1F)
const COLOR_MODE: u8 = 0xF0;


// led-server = tcp 192.168.10.131:5577
// response from a 5-channel LEDENET controller:
// pos  0  1  2  3  4  5  6  7  8  9 10 11 12 13
//    81 25 23 61 21 06 38 05 06 f9 01 00 0f 9d
//     |  |  |  |  |  |  |  |  |  |  |  |  |  |
//     |  |  |  |  |  |  |  |  |  |  |  |  |  checksum
//     |  |  |  |  |  |  |  |  |  |  |  |  color mode (f0 colors were set, 0f whites, 00 all were set)
//     |  |  |  |  |  |  |  |  |  |  |  cool-white  0x00 to 0xFF
//     |  |  |  |  |  |  |  |  |  |  version number
//     |  |  |  |  |  |  |  |  |  warmwhite  0x00 to 0xFF
//     |  |  |  |  |  |  |  |  blue  0x00 to 0xFF
//     |  |  |  |  |  |  |  green  0x00 to 0xFF
//     |  |  |  |  |  |  red 0x00 to 0xFF
//     |  |  |  |  |  speed: 0x01 = highest 0x1f is lowest
//     |  |  |  |  Mode WW(01), WW+CW(02), RGB(03), RGBW(04), RGBWW(05)
//     |  |  |  preset pattern
//     |  |  off(23)/on(24)
//     |  type
//     msg head
//

pub struct MagicHome {
    addr: String,
    stream: Option<TcpStream>,
    connected: bool,
    pub state: State,
    mode: Mode
}

impl MagicHome {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: String::from(addr),
            stream: None,
            connected: false,
            state: State::OFF,
            mode: Mode::INSTANT
        }
    }

    fn write_bytes(&mut self, mut buf: Vec<u8>) -> anyhow::Result<()> {
        let checksum = self.get_checksum(&buf);
        buf.push(checksum);
        if let Some(ref mut s) = self.stream {
            s.write(&buf)?;
        }
        Ok(())
    }

    fn get_checksum(&self, buf: &[u8]) -> u8 {
        buf.iter().fold(0u64, |a,b| a + (*b as u64)) as u8
    }
}

impl Output for MagicHome {
    fn connect(&mut self) -> anyhow::Result<()> {
        self.stream = Some(TcpStream::connect(&self.addr)?);
        self.connected = true;

        let mut query_buffer: Vec<u8> = vec![];
        query_buffer.push(0x81);
        query_buffer.push(0x8A);
        query_buffer.push(0x8B);
        self.write_bytes(query_buffer)?;
        if let Some(ref mut s) = self.stream {
            let mut buf: [u8; 14] = [0; 14];
            s.read(&mut buf)?;
            if buf[2] == 0x24 {
                self.state = State::OFF;
            } else {
                self.state = State::ON;
            }
        }
        

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn on_off(&mut self) -> anyhow::Result<()> {
        match self.stream {
            Some(_) => {
                match self.state {
                    State::ON => {
                        log::info!("Turning lights off");
                        let mut buf = vec![0x71, 0x24, 0x0F];
                        self.write_bytes(buf)?;
                        self.state = State::OFF;
                    },
                    State::OFF => {
                        log::info!("Turning lights on");
                        let mut buf = vec![0x71, 0x23, 0x0F];
                        self.write_bytes(buf)?;
                        self.state = State::ON;
                    }
                }
            }
            None => todo!(),
        }

        Ok(())
    }

    fn set_color(&mut self, rgb: [u8; 3]) -> anyhow::Result<()> {
        match self.stream {
            Some(_) => {
                let mut buf = vec![];
                match self.mode {
                    Mode::INSTANT => {
                        buf.push(CUSTOM_PATTERN_MODE);
                        for i in 0..rgb.len() {
                            buf.push(rgb[i]);
                        }
                        for _ in 0..15 {
                            buf.push(LIST_TERMINATOR);
                            buf.push(0x1);
                            buf.push(0x2);
                            buf.push(0x3);
                        }
                        buf.push(LIST_TERMINATOR);
                        buf.push(PATTERN_SPEED);
                        buf.push(JUMP_MODE);
                        buf.push(PATTERN_TERMINATOR_1);
                        buf.push(PATTERN_TERMINATOR_2);
                        self.write_bytes(buf)?;
                    },
                    Mode::FADE => {
                        buf.push(0x31);
                        for i in 0..rgb.len() {
                            buf.push(rgb[i]);
                        }
                        buf.push(LIST_TERMINATOR);
                        buf.push(COLOR_MODE);
                        buf.push(PATTERN_TERMINATOR_2);
                    }
                } 
                
                self.write_bytes(buf)?;
                self.state = State::ON;
            }
            None => todo!(),
        }

        Ok(())
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}
