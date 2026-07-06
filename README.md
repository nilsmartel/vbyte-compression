# vbyte

Variable-byte encoding for integers, designed to compresses values without loss.
The algorithm itself is implemented to compress/decompress unsigned 64-bit,
but other values that can be converted to (a subset of) unsigned 64 bit integers and converted back (from that subset) are fine to be used as well.


Values below 128 fit in a single byte. Each additional 7 bits of magnitude require one more byte, making the format efficient for sequences of small positive integers such as delta-encoded lists.

## Format

Each byte stores 7 bits of data. The most significant bit is a continuation flag: 1 if more bytes follow, 0 on the final byte. Bytes are ordered from least to most significant.

| Range                   | Bytes |
|-------------------------|-------|
| 0 – 127                 | 1     |
| 128 – 16 383            | 2     |
| 16 384 – 2 097 151      | 3     |
| 2 097 152 – 268 435 455 | 4     |

## Usage

```rust
use vbyte::{compress, decompress, compress_list, decompress_list};

let bytes = compress(1024);
let (value, _rest) = decompress(&bytes).unwrap();
assert_eq!(value, 1024);

let encoded = compress_list(&[1, 2, 3, 300]);
let values = decompress_list(&encoded).unwrap();
```
