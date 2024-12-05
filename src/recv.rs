// SPDX Licence-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2024 Laurent Fazio <laurent.fazio@gmail.com>

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

use super::statistics::CobsStatistics;

pub trait CobsReceiverOperation {
    fn recv(&mut self, len: usize) -> Option<Vec<u8>>;
}

pub struct CobsReceiver<'l> {
    stats: CobsStatistics,
    receiver: &'l Rc<RefCell<&'l mut dyn CobsReceiverOperation>>,
}

impl<'l> CobsReceiver<'l> {
    pub fn new(
        receiver: &'l Rc<RefCell<&'l mut dyn CobsReceiverOperation>>,
    ) -> CobsReceiver<'l> {
        CobsReceiver {
            stats: CobsStatistics::default(),
            receiver,
        }
    }

    pub fn stats(&self) -> &CobsStatistics {
        &self.stats
    }

    pub fn recv(&mut self) -> Option<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        let mut encoded: usize = 0;
        let mut code: usize = 0xff;
        let mut block: usize = 0x00;

        loop {
            if block > 0 {
                let mut buf = self.receiver.borrow_mut().recv(block)?;

                data.append(&mut buf);
                encoded += block;
                block = 0;
            } else {
                block = match self.receiver.borrow_mut().recv(1) {
                    Some(mut c) => c.pop()?,
                    None => 0,
                } as usize;

                encoded += 1;

                if block > 0 && code != 0xff {
                    data.push(0x00);
                }

                code = block;
                if code == 0 {
                    break;
                }

                block -= 1;
            }
        }

        self.stats.update(data.len(), encoded);

        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

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

    #[test]
    fn test_recv_00() {
        let pattern: Vec<u8> = vec![0x00];
        let encoded: &[u8] = &[0x01, 0x01, 0x00];

        let mut s2m = Mem2Recv::new(encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_00_00() {
        let pattern: Vec<u8> = vec![0x00, 0x00];
        let encoded: &[u8] = &[0x01, 0x01, 0x01, 0x00];

        let mut s2m = Mem2Recv::new(encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_00_11_00() {
        let pattern: Vec<u8> = vec![0x00, 0x11, 0x00];
        let encoded: Vec<u8> = vec![0x01, 0x02, 0x11, 0x01, 0x00];

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_11_22_00_33() {
        let pattern: Vec<u8> = vec![0x11, 0x22, 0x00, 0x33];
        let encoded: Vec<u8> = vec![0x03, 0x11, 0x22, 0x02, 0x33, 0x00];

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_11_22_33_44() {
        let pattern: Vec<u8> = vec![0x11, 0x22, 0x33, 0x44];
        let encoded: Vec<u8> = vec![0x05, 0x11, 0x22, 0x33, 0x44, 0x00];

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_11_00_00_00() {
        let pattern: Vec<u8> = vec![0x11, 0x00, 0x00, 0x00];
        let encoded: Vec<u8> = vec![0x02, 0x11, 0x01, 0x01, 0x01, 0x00];

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_long_0x01_0xfe() {
        let pattern: Vec<u8> = (1..=0xfe).collect::<Vec<_>>();
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.push(0x00);

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_long_0x00_0xfe() {
        let pattern: Vec<u8> = (0..=0xfe).collect::<Vec<_>>();
        let mut encoded: Vec<u8> = vec![0x01, 0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.push(0x00);

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_long_0x01_0xff() {
        let pattern: Vec<u8> = (1..=0xff).collect::<Vec<_>>();
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (1..=0xfe).collect::<Vec<_>>());
        encoded.append(&mut vec![0x02_u8, 0xff_u8, 0x00_u8]);

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_long_0x02_0xff_00() {
        let pattern: Vec<u8> = (2..=0x100)
            .map(|i: u16| (i % 0x100) as u8)
            .collect::<Vec<_>>();
        let mut encoded: Vec<u8> = vec![0xff];
        encoded.append(&mut (2..=0xff).collect::<Vec<_>>());
        encoded.append(&mut vec![0x01_u8, 0x01_u8, 0x00_u8]);

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }

    #[test]
    fn test_recv_long_0x03_0xff_00_01() {
        let pattern: Vec<u8> = (3..=0x101)
            .map(|i: u16| (i % 0x100) as u8)
            .collect::<Vec<_>>();
        let mut encoded: Vec<u8> = vec![0xfe];
        encoded.append(&mut (3..=0xff).collect::<Vec<_>>());
        encoded.append(&mut vec![0x02_u8, 0x01_u8, 0x00_u8]);

        let mut s2m = Mem2Recv::new(&encoded);
        let receiver: Rc<RefCell<&mut dyn CobsReceiverOperation>> = Rc::new(RefCell::new(&mut s2m));
        let mut r: CobsReceiver = CobsReceiver::new(&receiver);

        match r.recv() {
            Some(p) => {
                let (raw, enc) = r.stats().get();
                assert_eq!(raw, pattern.len());
                assert_eq!(enc, encoded.len());

                assert_eq!(p.len(), pattern.len());
                assert_eq!(p.cmp(&pattern), Ordering::Equal);
            }
            None => assert_eq!(false, true),
        }
    }
}
