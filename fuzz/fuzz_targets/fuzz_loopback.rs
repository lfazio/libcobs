#![no_main]

use libfuzzer_sys::fuzz_target;
use std::{cell::RefCell, rc::Rc};

extern crate libcobs;
use libcobs::{recv::{CobsReceiver, CobsReceiverOperation}, send::{CobsSender, CobsSenderOperation}};

use std::cmp::Ordering;

pub struct Send2Mem {
    pub data: Vec<u8>,
}

impl Send2Mem {
    pub fn new() -> Send2Mem {
        Send2Mem {
            data: vec![],
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl CobsSenderOperation for Send2Mem {
    fn send(&mut self, buf: &[u8]) -> Option<usize> {
        for v in buf {
            self.data.push(*v)
        }

        Some(buf.len())
    }
}

pub struct Mem2Recv<'l> {
    pub data: &'l [u8],
    pub offset: usize,
}

impl<'l> Mem2Recv<'l> {
    pub fn new(pattern: &[u8]) -> Mem2Recv {
        Mem2Recv {
            data: pattern,
            offset: 0,
        }
    }
}

impl<'l> CobsReceiverOperation for Mem2Recv<'l> {
    fn recv(&mut self, len: usize) -> Option<Vec<u8>> {
        let start = self.offset;

        let length = if self.offset + len > self.data.len() {
            self.data.len() - self.offset
        } else {
            len
        };

        if length == 0 {
            return None;
        }

        self.offset += length;

        Some(self.data[start..start + length].to_vec())
    }
}

fuzz_target!(|data: &[u8]| {
    let mut s2m = Send2Mem::new();
    let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
    let mut s: CobsSender = CobsSender::new(&sender);

    match s.send(data) {
        Some(_) => {
            let mut s2m = Mem2Recv::new(s2m.data());
            let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
            let mut r: CobsReceiver = CobsReceiver::new(&receiver);

            match r.recv() {
                Some(p) => {
                    assert_eq!(p.cmp(&data.to_vec()), Ordering::Equal);
                }
                None => assert_eq!(false, true),
            }
        }
        None => assert_eq!(false, true),
    }
});
