//! Symbol frequency analysis for Huffman coding.

use std::collections::HashMap;

/// A frequency table mapping symbols to their occurrence counts.
#[derive(Debug, Clone, Default)]
pub struct FrequencyTable {
    counts: HashMap<u8, usize>,
    total: usize,
}

impl FrequencyTable {
    /// Create an empty frequency table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a frequency table from a slice of bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut table = Self::new();
        for &byte in data {
            table.increment(byte);
        }
        table
    }

    /// Increment the count for a symbol.
    pub fn increment(&mut self, symbol: u8) {
        *self.counts.entry(symbol).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get the count for a symbol.
    pub fn count(&self, symbol: u8) -> usize {
        self.counts.get(&symbol).copied().unwrap_or(0)
    }

    /// Get the total number of symbols counted.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get an iterator over all (symbol, count) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&u8, &usize)> {
        self.counts.iter()
    }

    /// Get the number of unique symbols.
    pub fn unique_count(&self) -> usize {
        self.counts.len()
    }

    /// Get the probability of a symbol.
    pub fn probability(&self, symbol: u8) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.count(symbol) as f64 / self.total as f64
        }
    }

    /// Merge another frequency table into this one.
    pub fn merge(&mut self, other: &FrequencyTable) {
        for (&symbol, &count) in other.iter() {
            *self.counts.entry(symbol).or_insert(0) += count;
            self.total += count;
        }
    }

    /// Get the most frequent symbol.
    pub fn most_frequent(&self) -> Option<u8> {
        self.counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&symbol, _)| symbol)
    }

    /// Get the least frequent symbol.
    pub fn least_frequent(&self) -> Option<u8> {
        self.counts
            .iter()
            .min_by_key(|(_, &count)| count)
            .map(|(&symbol, _)| symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_table() {
        let table = FrequencyTable::new();
        assert_eq!(table.total(), 0);
        assert_eq!(table.unique_count(), 0);
        assert_eq!(table.count(0), 0);
    }

    #[test]
    fn test_from_bytes_simple() {
        let data = b"aab";
        let table = FrequencyTable::from_bytes(data);
        assert_eq!(table.count(b'a'), 2);
        assert_eq!(table.count(b'b'), 1);
        assert_eq!(table.count(b'c'), 0);
        assert_eq!(table.total(), 3);
        assert_eq!(table.unique_count(), 2);
    }

    #[test]
    fn test_probability() {
        let data = b"aabb";
        let table = FrequencyTable::from_bytes(data);
        assert!((table.probability(b'a') - 0.5).abs() < f64::EPSILON);
        assert!((table.probability(b'b') - 0.5).abs() < f64::EPSILON);
        assert!((table.probability(b'c')).abs() < f64::EPSILON);
    }

    #[test]
    fn test_most_least_frequent() {
        let data = b"aaabbc";
        let table = FrequencyTable::from_bytes(data);
        assert_eq!(table.most_frequent(), Some(b'a'));
        assert_eq!(table.least_frequent(), Some(b'c'));
    }

    #[test]
    fn test_merge() {
        let mut t1 = FrequencyTable::from_bytes(b"aa");
        let t2 = FrequencyTable::from_bytes(b"abb");
        t1.merge(&t2);
        assert_eq!(t1.count(b'a'), 3);
        assert_eq!(t1.count(b'b'), 2);
        assert_eq!(t1.total(), 5);
    }
}
