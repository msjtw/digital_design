use std::os::fd::AsFd;
use std::thread;
use std::time::Duration;

use std::io::{self, Read};
use termion;

fn main() {
    loop {
        termion::input::Keys::count();
        let mut buff: [u8; 1] = [0; 1];
        let rv = io::stdin().read(&mut buff);
        println!("buffered: {:?}, bytes", rv);
        thread::sleep(Duration::from_millis(500));
    }
}
