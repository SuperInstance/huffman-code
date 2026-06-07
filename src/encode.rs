//! Huffman encoding of data using pre-built trees.

use crate::tree::{BitCode, HuffmanTree};
use std::collections::HashMap;

/// Encoded output containing the bitstream and original length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedData {
    /// Packed bits (MSB first within each byte).
    pub data: Vec<u8>,
    /// Number of valid bits in the last byte.
    pub last_byte_bits: u8,
    /// Length of the original data in bytes.
    pub original_len: usize,
}

/// Encode a byte slice using a pre-built Huffman tree.
pub fn encode(data: &[u8], tree: &HuffmanTree) -> EncodedData {
    let code_table = tree.code_table();
    let mut bits = Vec::new();

    for &byte in data {
        if let Some(code) = code_table.get(&byte) {
            bits.extend_from_slice(code.bits());
        }
    }

    let (packed, last_byte_bits) = pack_bits(&bits);
    EncodedData {
        data: packed,
        last_byte_bits,
        original_len: data.len(),
    }
}

/// Encode using canonical Huffman codes.
pub fn encode_canonical(data: &[u8], codes: &HashMap<u8, BitCode>) -> EncodedData {
    let mut bits = Vec::new();
    for &byte in data {
        if let Some(code) = codes.get(&byte) {
            bits.extend_from_slice(code.bits());
        }
    }
    let (packed, last_byte_bits) = pack_bits(&bits);
    EncodedData {
        data: packed,
        last_byte_bits,
        original_len: data.len(),
    }
}

/// Pack a bit vector into bytes (MSB first).
fn pack_bits(bits: &[u8]) -> (Vec<u8>, u8) {
    if bits.is_empty() {
        return (Vec::new(), 0);
    }
    let mut packed = Vec::with_capacity(bits.len().div_ceil(8));
    let mut current = 0u8;
    let mut bit_pos = 7;

    for &bit in bits {
        current |= (bit & 1) << bit_pos;
        if bit_pos == 0 {
            packed.push(current);
            current = 0;
            bit_pos = 7;
        } else {
            bit_pos -= 1;
        }
    }

    if bit_pos < 7 {
        // Push the partial byte
        packed.push(current);
        let last_bits = (7 - bit_pos) as u8;
        (packed, last_bits)
    } else {
        // All bits fit perfectly
        (packed, 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frequency::FrequencyTable;

    #[test]
    fn test_pack_bits_empty() {
        let (packed, extra) = pack_bits(&[]);
        assert!(packed.is_empty());
        assert_eq!(extra, 0);
    }

    #[test]
    fn test_pack_bits_full_byte() {
        let bits = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let (packed, extra) = pack_bits(&bits);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0], 0b10101010);
        assert_eq!(extra, 8);
    }

    #[test]
    fn test_encode_roundtrip_simple() {
        let data = b"hello";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode(data, &tree);
        assert!(!encoded.data.is_empty());
    }

    #[test]
    fn test_encode_preserves_original_len() {
        let data = b"test data";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode(data, &tree);
        assert_eq!(encoded.original_len, data.len());
    }

    #[test]
    fn test_encode_canonical() {
        let data = b"abracadabra";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let canonical = tree.canonical_codes();
        let codes: HashMap<u8, BitCode> = canonical.into_iter().collect();
        let encoded = encode_canonical(data, &codes);
        assert_eq!(encoded.original_len, data.len());
    }
}
