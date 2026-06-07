//! Huffman tree construction and traversal.

use std::collections::{BTreeMap, HashMap};
use crate::frequency::FrequencyTable;

/// A bit sequence for Huffman codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitCode {
    bits: Vec<u8>, // each bit is 0 or 1
}

impl Default for BitCode {
    fn default() -> Self {
        Self::new()
    }
}

impl BitCode {
    pub fn new() -> Self {
        Self { bits: Vec::new() }
    }

    pub fn push(&mut self, bit: u8) {
        self.bits.push(bit & 1);
    }

    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    pub fn from_bits(bits: Vec<u8>) -> Self {
        Self { bits }
    }
}

/// A node in the Huffman tree.
#[derive(Debug, Clone)]
pub enum HuffmanNode {
    Leaf {
        symbol: u8,
        weight: usize,
    },
    Internal {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
        weight: usize,
    },
}

impl HuffmanNode {
    pub fn weight(&self) -> usize {
        match self {
            HuffmanNode::Leaf { weight, .. } => *weight,
            HuffmanNode::Internal { weight, .. } => *weight,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, HuffmanNode::Leaf { .. })
    }

    /// Get the symbol if this is a leaf node.
    pub fn symbol(&self) -> Option<u8> {
        match self {
            HuffmanNode::Leaf { symbol, .. } => Some(*symbol),
            HuffmanNode::Internal { .. } => None,
        }
    }
}

/// A Huffman tree.
#[derive(Debug, Clone)]
pub struct HuffmanTree {
    root: HuffmanNode,
}

impl HuffmanTree {
    /// Build a Huffman tree from a frequency table.
    pub fn from_frequency_table(freq: &FrequencyTable) -> Self {
        let mut nodes: Vec<HuffmanNode> = freq
            .iter()
            .map(|(&symbol, &weight)| HuffmanNode::Leaf { symbol, weight })
            .collect();

        if nodes.is_empty() {
            return Self {
                root: HuffmanNode::Leaf {
                    symbol: 0,
                    weight: 0,
                },
            };
        }

        if nodes.len() == 1 {
            let node = nodes.pop().unwrap();
            return Self {
                root: HuffmanNode::Internal {
                    left: Box::new(node),
                    right: Box::new(HuffmanNode::Leaf {
                        symbol: 0,
                        weight: 0,
                    }),
                    weight: 0,
                },
            };
        }

        while nodes.len() > 1 {
            nodes.sort_by_key(|b| std::cmp::Reverse(b.weight()));
            let right = nodes.pop().unwrap();
            let left = nodes.pop().unwrap();
            let weight = left.weight() + right.weight();
            nodes.push(HuffmanNode::Internal {
                left: Box::new(left),
                right: Box::new(right),
                weight,
            });
        }

        Self {
            root: nodes.pop().unwrap(),
        }
    }

    /// Get a reference to the root node.
    pub fn root(&self) -> &HuffmanNode {
        &self.root
    }

    /// Generate code table: symbol -> bit sequence.
    pub fn code_table(&self) -> HashMap<u8, BitCode> {
        let mut table = HashMap::new();
        Self::build_codes(&self.root, &BitCode::new(), &mut table);
        table
    }

    fn build_codes(
        node: &HuffmanNode,
        prefix: &BitCode,
        table: &mut HashMap<u8, BitCode>,
    ) {
        match node {
            HuffmanNode::Leaf { symbol, .. } => {
                table.insert(*symbol, prefix.clone());
            }
            HuffmanNode::Internal { left, right, .. } => {
                let mut left_code = prefix.clone();
                left_code.push(0);
                Self::build_codes(left, &left_code, table);

                let mut right_code = prefix.clone();
                right_code.push(1);
                Self::build_codes(right, &right_code, table);
            }
        }
    }

    /// Generate canonical Huffman codes from this tree.
    /// Canonical codes are sorted by code length, then by symbol value.
    pub fn canonical_codes(&self) -> BTreeMap<u8, BitCode> {
        let table = self.code_table();
        let mut by_length: Vec<(usize, u8)> = table
            .iter()
            .map(|(&sym, code)| (code.len(), sym))
            .collect();
        by_length.sort();

        let mut canonical = BTreeMap::new();
        let mut code_val: u64 = 0;
        let mut prev_len: usize = 0;

        for (len, symbol) in by_length {
            code_val <<= len - prev_len;
            let bits: Vec<u8> = (0..len)
                .rev()
                .map(|i| ((code_val >> i) & 1) as u8)
                .collect();
            canonical.insert(symbol, BitCode::from_bits(bits));
            code_val += 1;
            prev_len = len;
        }

        canonical
    }

    /// Compute the weighted path length (total bits needed).
    pub fn weighted_path_length(&self) -> usize {
        Self::wpl(&self.root, 0)
    }

    fn wpl(node: &HuffmanNode, depth: usize) -> usize {
        match node {
            HuffmanNode::Leaf { weight, .. } => weight * depth,
            HuffmanNode::Internal { left, right, .. } => {
                Self::wpl(left, depth + 1) + Self::wpl(right, depth + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_symbol_tree() {
        let freq = FrequencyTable::from_bytes(b"aaaa");
        let tree = HuffmanTree::from_frequency_table(&freq);
        let codes = tree.code_table();
        assert!(codes.contains_key(&b'a'));
    }

    #[test]
    fn test_two_symbol_tree() {
        let freq = FrequencyTable::from_bytes(b"ab");
        let tree = HuffmanTree::from_frequency_table(&freq);
        let codes = tree.code_table();
        assert_eq!(codes.len(), 2);
        // Both codes should be 1 bit
        for code in codes.values() {
            assert_eq!(code.len(), 1);
        }
    }

    #[test]
    fn test_frequent_symbol_shorter_code() {
        let freq = FrequencyTable::from_bytes(b"aaaaab");
        let tree = HuffmanTree::from_frequency_table(&freq);
        let codes = tree.code_table();
        // 'a' should have a shorter or equal code to 'b'
        assert!(codes[&b'a'].len() <= codes[&b'b'].len());
    }

    #[test]
    fn test_canonical_codes_sorted_by_length() {
        let data = b"aaabbc";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let canonical = tree.canonical_codes();
        // All symbols should have codes
        assert!(canonical.contains_key(&b'a'));
        assert!(canonical.contains_key(&b'b'));
        assert!(canonical.contains_key(&b'c'));
    }

    #[test]
    fn test_weighted_path_length() {
        let freq = FrequencyTable::from_bytes(b"aaabbc");
        let tree = HuffmanTree::from_frequency_table(&freq);
        let wpl = tree.weighted_path_length();
        assert!(wpl > 0);
    }
}
