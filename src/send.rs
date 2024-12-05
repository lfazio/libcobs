// SPDX Licence-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2024 Laurent Fazio <laurent.fazio@gmail.com>

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use super::statistics::CobsStatistics;

pub trait CobsSenderOperation {
    fn send(&mut self, buf: &[u8]) -> Option<usize>;
}

pub struct CobsSender<'l> {
    stats: CobsStatistics,
    sender: &'l Rc<RefCell<&'l mut dyn CobsSenderOperation>>,
}

impl<'l> CobsSender<'l> {
    pub fn new(
        sender: &'l Rc<RefCell<&'l mut dyn CobsSenderOperation>>,
    ) -> CobsSender<'l> {
        CobsSender {
            stats: CobsStatistics::default(),
            sender,
        }
    }

    pub fn stats(&self) -> &CobsStatistics {
        &self.stats
    }

    pub fn send(&mut self, buf: &[u8]) -> Option<usize> {
        let mut code: u8;
        let mut total: usize = 0;
        let mut i = 0;
        let mut start: usize;

        loop {
            code = 0x01;
            start = i;

            loop {
                if i >= buf.len() {
                    break;
                }

                if buf[i] == 0 {
                    break;
                }

                if code == 0xff {
                    break;
                }

                code += 1;
                i += 1;
            }

            self.sender.borrow_mut().send(&[code])?;

            if code > 0x01 {
                let end = i;
                let data = &buf[start..end];
                self.sender.borrow_mut().send(data)?;
            }

            total += code as usize;

            if i >= buf.len() {
                break;
            }

            if buf[i] == 0 && code < 0xff {
                i += 1;
            }
        }

        self.sender.borrow_mut().send(&[0])?;
        total += 1;

        self.stats.update(buf.len(), total);

        Some(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    pub struct Send2Mem {
        pub data: Vec<u8>,
    }

    impl Send2Mem {
        pub fn new() -> Send2Mem {
            Send2Mem { data: vec![] }
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

    #[test]
    fn test_send_00() {
        let pattern: [u8; 1] = [0x00];
        let encoded: Vec<u8> = vec![0x01, 0x01, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data().cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_00_00() {
        let pattern: [u8; 2] = [0x00, 0x00];
        let encoded: Vec<u8> = vec![0x01, 0x01, 0x01, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_00_11_00() {
        let pattern: [u8; 3] = [0x00, 0x11, 0x00];
        let encoded: Vec<u8> = vec![0x01, 0x02, 0x11, 0x01, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_11_22_00_33() {
        let pattern: [u8; 4] = [0x11, 0x22, 0x00, 0x33];
        let encoded: Vec<u8> = vec![0x03, 0x11, 0x22, 0x02, 0x33, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_11_22_33_44() {
        let pattern: [u8; 4] = [0x11, 0x22, 0x33, 0x44];
        let encoded: Vec<u8> = vec![0x05, 0x11, 0x22, 0x33, 0x44, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_11_00_00_00() {
        let pattern: [u8; 4] = [0x11, 0x00, 0x00, 0x00];
        let encoded: Vec<u8> = vec![0x02, 0x11, 0x01, 0x01, 0x01, 0x00];

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_long_0x01_0xfe() {
        let pattern: [u8; 254] = (1..=0xfe).collect::<Vec<_>>().try_into().expect("");
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.push(0x00);

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_long_0x00_0xfe() {
        let pattern: [u8; 255] = (0..=0xfe).collect::<Vec<_>>().try_into().expect("");
        let mut encoded: Vec<u8> = vec![0x01, 0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.push(0x00);

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_long_0x01_0xff() {
        let pattern: [u8; 255] = (1..=0xff).collect::<Vec<_>>().try_into().expect("");
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.append(&mut vec![0x02_u8, 0xff_u8, 0x00_u8]);

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_long_0x02_0xff_00() {
        let pattern: [u8; 255] = (2..=0x100)
            .map(|i: u16| (i % 0x100) as u8)
            .collect::<Vec<_>>()
            .try_into()
            .expect("");
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (2..=0xff).collect::<Vec<_>>());
        encoded.append(&mut vec![0x01_u8, 0x01_u8, 0x00_u8]);

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_send_long_0x03_0xff_00_01() {
        let pattern: [u8; 255] = (3..=0x101)
            .map(|i: u16| (i % 0x100) as u8)
            .collect::<Vec<_>>()
            .try_into()
            .expect("");
        let mut encoded: Vec<u8> = vec![0xfe];
        encoded.append(&mut (3..=0xff).collect::<Vec<_>>());
        encoded.append(&mut vec![0x02_u8, 0x01_u8, 0x00_u8]);

        let mut s2m = Send2Mem::new();
        let sender: Rc<RefCell<&mut dyn CobsSenderOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut s: CobsSender = CobsSender::new(&sender);

        match s.send(&pattern) {
            Some(l) => {
                let (raw, enc) = s.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(l, encoded.len());
                assert_eq!(s2m.data.cmp(&encoded), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }
}
