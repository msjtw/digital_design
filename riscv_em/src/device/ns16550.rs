use std::io::{Bytes, Read, Stdout, Write};
use termion::async_stdin;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Uart {
    base: u32,
    length: u32,

    stdin: Bytes<termion::AsyncReader>,
    stdout: RawTerminal<Stdout>,
    bytes_to_read: u8,

    dll: u8,
    dlh: u8,
    rhr: u8, // 0x0 r  receiver holding register
    thr: u8, // 0x0 w  transmiter holding register
    ier: u8, // 0x1 rw interrupt enable register
    iir: u8, // 0x2 r  interrupt identification register
    fcr: u8, // 0x2 w  FIFO control register
    lcr: u8, // 0x3 rw line control register
    lsr: u8, // 0x5 r  line status resister

    thr_interrupt: bool,
    rhr_interrupt: bool,
}

impl Default for Uart {
    fn default() -> Self {
        Uart {
            base: 0x10000000,
            length: 0x10,
            stdin: async_stdin().bytes(),
            stdout: std::io::stdout().into_raw_mode().unwrap(),
            bytes_to_read: 0,
            dll: 0,
            dlh: 0,
            rhr: 0,
            thr: 0,
            ier: 0,
            iir: 0,
            fcr: 0,
            lcr: 0,
            lsr: 0x60,
            thr_interrupt: false,
            rhr_interrupt: false,
        }
    }
}

impl Uart {
    pub fn claim(&self, addr: u32) -> bool {
        if addr >= self.base && addr < self.base + self.length {
            return true;
        }
        return false;
    }

    pub fn tick(&mut self) {
        if self.bytes_to_read == 0 {
            if let Some(Ok(byte)) = self.stdin.next() {
                if byte == 1 {
                    let mut next_byte: [u8; 1] = [0];
                    std::io::stdin().read_exact(&mut next_byte).unwrap();
                    if next_byte[0] == 3 {
                        std::process::exit(1);
                    }
                }
                self.rhr = byte;
                self.bytes_to_read = 1;
            }
        }

        if self.bytes_to_read > 0 {
            self.rhr_interrupt = true;
        }

        if self.rhr_interrupt && (self.ier & 0b1 != 0) {
            self.iir = 0b0100;
        } else if self.thr_interrupt && (self.ier & 0b10 != 0) {
            self.iir = 0b0010;
        } else {
            self.iir = 1;
        }

        if self.iir != 1 {
            // signal interrupt;
        }
    }

    pub fn write(&mut self, addr: u32, data: u8) {
        let addr = addr - self.base;
        match addr {
            // thr transmiter holding register
            0 => {
                if self.lcr & (1 << 7) != 0 {
                    self.dll = data;
                } else {
                    write!(self.stdout, "{}", data as char).unwrap();
                    self.stdout.flush().unwrap();
                    self.thr_interrupt = true;
                }
            }
            // ier interrupt enable register
            1 => {
                if self.lcr & (1 << 7) != 0 {
                    self.dlh = data;
                } else {
                    self.ier = data;
                }
            }
            // fcr FIFO control register
            2 => {
                self.fcr = data;
            }
            // lcr line control register
            3 => {
                self.lcr = data;
            }
            _ => {}
        }
    }

    pub fn read(&mut self, addr: u32) -> u8 {
        let addr = addr - self.base;
        return match addr {
            // rhr register holding register
            0 => {
                if self.lcr & (1 << 7) != 0 {
                    self.dll
                } else {
                    self.rhr_interrupt = false;
                    self.bytes_to_read = 0;
                    self.rhr
                }
            }
            // ier interrupt enable register
            1 => {
                if self.lcr & (1 << 7) != 0 {
                    self.dlh
                } else {
                    self.ier
                }
            }
            // iir interrupt identification register
            2 => self.iir,
            // lcr line control register
            3 => {
                if !self.rhr_interrupt {
                    self.thr_interrupt = false;
                }
                self.lcr
            }
            // lsr line status register
            5 => self.lsr | self.bytes_to_read,
            _ => 0,
        };
    }
}
