//! Adaptive Huffman coding (FGK algorithm).

const MAX_SYMBOLS: usize = 256;

/// A node in the adaptive Huffman tree.
#[derive(Debug, Clone)]
struct AdaptiveNode {
    #[allow(dead_code)]
    symbol: Option<u8>,
    weight: usize,
    #[allow(dead_code)]
    number: usize, // implicit numbering for sibling property
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

/// Adaptive Huffman tree (FGK algorithm).
#[derive(Debug, Clone)]
pub struct AdaptiveHuffmanTree {
    nodes: Vec<AdaptiveNode>,
    nyt_node: usize, // Not Yet Transmitted node
    symbol_to_node: [Option<usize>; MAX_SYMBOLS],
    next_number: usize,
}

impl Default for AdaptiveHuffmanTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveHuffmanTree {
    /// Create a new adaptive Huffman tree with NYT as the only node.
    pub fn new() -> Self {
        let nyt = AdaptiveNode {
            symbol: None,
            weight: 0,
            number: 0,
            left: None,
            right: None,
            parent: None,
        };
        let mut _symbol_map: [Option<usize>; MAX_SYMBOLS] = [None; MAX_SYMBOLS];
        Self {
            nodes: vec![nyt],
            nyt_node: 0,
            symbol_to_node: [None; MAX_SYMBOLS],
            next_number: 1,
        }
    }

    /// Get the code for a symbol if it has been seen before.
    pub fn get_code(&self, symbol: u8) -> Option<Vec<u8>> {
        let node_idx = self.symbol_to_node[symbol as usize]?;
        Some(self.code_for_node(node_idx))
    }

    /// Get the NYT (Not Yet Transmitted) code.
    pub fn get_nyt_code(&self) -> Vec<u8> {
        self.code_for_node(self.nyt_node)
    }

    /// Check if a symbol has been seen.
    pub fn has_seen(&self, symbol: u8) -> bool {
        self.symbol_to_node[symbol as usize].is_some()
    }

    fn code_for_node(&self, mut node_idx: usize) -> Vec<u8> {
        let mut bits = Vec::new();
        while let Some(parent_idx) = self.nodes[node_idx].parent {
            let parent = &self.nodes[parent_idx];
            if parent.left == Some(node_idx) {
                bits.push(0);
            } else {
                bits.push(1);
            }
            node_idx = parent_idx;
        }
        bits.reverse();
        bits
    }

    /// Update the tree with a new symbol (encode side).
    /// Returns the code for the symbol: NYT code + raw bits if first occurrence.
    pub fn encode_symbol(&mut self, symbol: u8) -> Vec<u8> {
        if self.has_seen(symbol) {
            let code = self.get_code(symbol).unwrap();
            self.update(symbol);
            code
        } else {
            // First occurrence: send NYT code + 8-bit raw symbol
            let mut code = self.get_nyt_code();
            for i in (0..8).rev() {
                code.push((symbol >> i) & 1);
            }
            self.update(symbol);
            code
        }
    }

    /// Update the tree after processing a symbol.
    fn update(&mut self, symbol: u8) {
        let node_idx = if self.has_seen(symbol) {
            self.symbol_to_node[symbol as usize].unwrap()
        } else {
            // Split NYT node
            let nyt = self.nyt_node;
            let new_internal = AdaptiveNode {
                symbol: None,
                weight: 0,
                number: self.next_number,
                left: Some(self.nyt_node),
                right: None,
                parent: self.nodes[nyt].parent,
            };
            self.next_number += 1;
            let internal_idx = self.nodes.len();
            self.nodes.push(new_internal);

            let new_nyt = AdaptiveNode {
                symbol: None,
                weight: 0,
                number: self.next_number,
                left: None,
                right: None,
                parent: Some(internal_idx),
            };
            self.next_number += 1;
            let new_nyt_idx = self.nodes.len();
            self.nodes.push(new_nyt);

            let leaf = AdaptiveNode {
                symbol: Some(symbol),
                weight: 0,
                number: self.next_number,
                left: None,
                right: None,
                parent: Some(internal_idx),
            };
            self.next_number += 1;
            let leaf_idx = self.nodes.len();
            self.nodes.push(leaf);

            self.nodes[internal_idx].left = Some(new_nyt_idx);
            self.nodes[internal_idx].right = Some(leaf_idx);

            // Update parent references
            if let Some(parent) = self.nodes[nyt].parent {
                if self.nodes[parent].left == Some(nyt) {
                    self.nodes[parent].left = Some(internal_idx);
                } else {
                    self.nodes[parent].right = Some(internal_idx);
                }
            }

            self.nodes[nyt].parent = Some(internal_idx);
            self.symbol_to_node[symbol as usize] = Some(leaf_idx);
            self.nyt_node = new_nyt_idx;

            // Increment weight up the tree
            self.increment_weights(leaf_idx);
            return;
        };

        self.increment_weights(node_idx);
    }

    fn increment_weights(&mut self, mut node_idx: usize) {
        loop {
            self.nodes[node_idx].weight += 1;
            match self.nodes[node_idx].parent {
                Some(parent) => node_idx = parent,
                None => break,
            }
        }
    }

    /// Encode an entire byte slice adaptively.
    pub fn encode_all(data: &[u8]) -> Vec<u8> {
        let mut tree = Self::new();
        let mut bits = Vec::new();
        for &symbol in data {
            let code = tree.encode_symbol(symbol);
            bits.extend(code);
        }
        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_new_tree() {
        let tree = AdaptiveHuffmanTree::new();
        assert!(!tree.has_seen(b'a'));
        assert!(!tree.has_seen(b'b'));
    }

    #[test]
    fn test_adaptive_first_symbol() {
        let mut tree = AdaptiveHuffmanTree::new();
        let code = tree.encode_symbol(b'a');
        assert!(tree.has_seen(b'a'));
        // First symbol: NYT code (empty for root) + 8 raw bits = 8 bits
        assert_eq!(code.len(), 8);
    }

    #[test]
    fn test_adaptive_second_symbol() {
        let mut tree = AdaptiveHuffmanTree::new();
        tree.encode_symbol(b'a');
        let code = tree.encode_symbol(b'a');
        // Second occurrence: should be short code
        assert!(code.len() <= 3);
    }

    #[test]
    fn test_adaptive_encode_all() {
        let data = b"aabbc";
        let bits = AdaptiveHuffmanTree::encode_all(data);
        // Should produce some bits
        assert!(!bits.is_empty());
    }

    #[test]
    fn test_adaptive_updates_seen() {
        let mut tree = AdaptiveHuffmanTree::new();
        tree.encode_symbol(b'x');
        tree.encode_symbol(b'y');
        tree.encode_symbol(b'z');
        assert!(tree.has_seen(b'x'));
        assert!(tree.has_seen(b'y'));
        assert!(tree.has_seen(b'z'));
        assert!(!tree.has_seen(b'w'));
    }

    #[test]
    fn test_adaptive_repeated_same_symbol() {
        let data = b"aaaaa";
        let bits = AdaptiveHuffmanTree::encode_all(data);
        // First: 8 bits, subsequent symbols should have short codes
        // Total should be much less than 5*8 = 40 bits
        assert!(bits.len() < 30);
    }
}
