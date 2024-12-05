// SPDX Licence-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2024 Laurent Fazio <laurent.fazio@gmail.com>

#[derive(Default)]
pub struct CobsStatistics {
    raw: usize,
    encoded: usize,
}

impl CobsStatistics {
    pub fn update(&mut self, raw: usize, encoded: usize) {
        self.raw += raw;
        self.encoded += encoded;
    }

    pub fn get(&self) -> (usize, usize) {
        (self.raw, self.encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_update() {
        let mut s = CobsStatistics::default();

        s.update(1, 1);
        assert_eq!(s.get(), (1, 1));
        s.update(1, 1);
        assert_eq!(s.get(), (2, 2));

        s.update(1, 1);
        assert_eq!(s.get(), (3, 3));
        s.update(1, 1);
        assert_eq!(s.get(), (4, 4));
    }
}