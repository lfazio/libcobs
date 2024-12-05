use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct CobsStatistics {
    raw: usize,
    encoded: usize,
}

impl CobsStatistics {
    pub fn new() -> CobsStatistics {
        CobsStatistics { raw: 0, encoded: 0 }
    }

    pub fn update(&mut self, raw: usize, encoded: usize) {
        self.raw += raw;
        self.encoded += encoded;
    }

    pub fn get(&self) -> (usize, usize) {
        (self.raw, self.encoded)
    }
}

impl fmt::Display for CobsStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<raw={}, encoded={}>", self.raw, self.encoded)
    }
}
