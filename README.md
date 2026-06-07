# huffman-code

A pure-Rust library for Huffman coding — frequency analysis, tree construction, canonical codes, and adaptive Huffman encoding/decoding.

## Features

- **Frequency analysis** — Build frequency tables from byte data
- **Tree construction** — Build optimal Huffman trees from frequency data
- **Canonical Huffman codes** — Generate canonical code representations for efficient storage
- **Adaptive Huffman** — Dynamically update the tree during encoding/decoding
- **Roundtrip encode/decode** — Lossless compression and decompression

## Usage

```rust
use huffman_code::{frequency::FrequencyTable, tree::HuffmanTree, encode, decode};

let data = b"hello world";
let freq = FrequencyTable::from_bytes(data);
let tree = HuffmanTree::from_frequency_table(&freq);
let encoded = encode::encode(data, &tree);
let decoded = decode::decode(&encoded, &tree).unwrap();
assert_eq!(data.as_slice(), decoded.as_slice());
```

## Modules

- `frequency` — Symbol frequency analysis
- `tree` — Huffman tree construction and traversal
- `encode` — Encoding data using Huffman codes
- `decode` — Decoding Huffman-encoded bitstreams
- `adaptive` — Adaptive Huffman coding (FGK algorithm)

## License

MIT
