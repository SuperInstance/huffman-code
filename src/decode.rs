//! Huffman decoding of encoded bitstreams.

use crate::encode::EncodedData;
use crate::tree::{HuffmanNode, HuffmanTree};

/// Decode Huffman-encoded data back to the original bytes.
pub fn decode(encoded: &EncodedData, tree: &HuffmanTree) -> Result<Vec<u8>, String> {
    if encoded.original_len == 0 {
        return Ok(Vec::new());
    }

    let bits = unpack_bits(&encoded.data, encoded.last_byte_bits);
    let mut result = Vec::with_capacity(encoded.original_len);
    let mut current = tree.root();

    for bit in &bits {
        current = match (bit, current) {
            (0, HuffmanNode::Internal { left, .. }) => left.as_ref(),
            (1, HuffmanNode::Internal { right, .. }) => right.as_ref(),
            (_, HuffmanNode::Internal { left, .. }) => left.as_ref(), // bits > 1 treated as 0
            (_, HuffmanNode::Leaf { .. }) => {
                return Err("Unexpected leaf during traversal".to_string());
            }
        };

        if let HuffmanNode::Leaf { symbol, .. } = current {
            result.push(*symbol);
            if result.len() == encoded.original_len {
                break;
            }
            current = tree.root();
        }
    }

    if result.len() != encoded.original_len {
        return Err(format!(
            "Decoded length mismatch: expected {}, got {}",
            encoded.original_len,
            result.len()
        ));
    }

    Ok(result)
}

/// Unpack bytes back to individual bits.
fn unpack_bits(data: &[u8], last_byte_bits: u8) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut bits = Vec::new();
    for (i, &byte) in data.iter().enumerate() {
        let valid_bits = if i == data.len() - 1 && last_byte_bits > 0 && last_byte_bits < 8 {
            last_byte_bits as usize
        } else {
            8
        };
        for j in 0..valid_bits {
            bits.push((byte >> (7 - j)) & 1);
        }
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;
    use crate::frequency::FrequencyTable;

    #[test]
    fn test_decode_empty() {
        let encoded = encode::EncodedData {
            data: Vec::new(),
            last_byte_bits: 0,
            original_len: 0,
        };
        let freq = FrequencyTable::new();
        let tree = HuffmanTree::from_frequency_table(&freq);
        let decoded = decode(&encoded, &tree).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_simple_roundtrip() {
        let data = b"hello";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode::encode(data, &tree);
        let decoded = decode(&encoded, &tree).unwrap();
        assert_eq!(decoded, data.to_vec());
    }

    #[test]
    fn test_decode_longer_roundtrip() {
        let data = b"the quick brown fox jumps over the lazy dog";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode::encode(data, &tree);
        let decoded = decode(&encoded, &tree).unwrap();
        assert_eq!(decoded, data.to_vec());
    }

    #[test]
    fn test_decode_single_byte_roundtrip() {
        let data = b"a";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode::encode(data, &tree);
        let decoded = decode(&encoded, &tree).unwrap();
        assert_eq!(decoded, data.to_vec());
    }

    #[test]
    fn test_decode_repeated_roundtrip() {
        let data = b"aaaaaaaabbbcc";
        let freq = FrequencyTable::from_bytes(data);
        let tree = HuffmanTree::from_frequency_table(&freq);
        let encoded = encode::encode(data, &tree);
        let decoded = decode(&encoded, &tree).unwrap();
        assert_eq!(decoded, data.to_vec());
    }
}
