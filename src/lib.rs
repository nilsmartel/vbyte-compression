use std::io::Write;
pub mod utils;

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! single_test {
        ($n:ident, $i:expr) => {
            #[test]
            fn $n() {
                let i = $i;
                let c = compress(i);
                let (d, rest) = decompress(&c).unwrap();
                assert_eq!(i, d);
                assert!(rest.is_empty(), "no rest should be remaining.");
            }
        };
    }

    #[test]
    fn simple1() {
        let i = 36;
        let c = compress(i);
        let (d, rest) = decompress(&c).unwrap();
        assert_eq!(i, d);
        assert!(rest.is_empty(), "no rest should be remaining.");
    }

    single_test!(simple0, 0);
    single_test!(simple2, 2);
    single_test!(simple3, 3);
    single_test!(simple32, 32);
    single_test!(simple64, 64);
    single_test!(simple65, 65);
    single_test!(simple127, 127);
    single_test!(simple128, 128);
    single_test!(simple244, 244);
    single_test!(simple12341234, 12341234);

    #[test]
    fn test_set() {
        let ints = [
            2134123213213u64,
            2313,
            3,
            3213,
            21321,
            3213,
            213,
            213,
            5435,
            5654,
            6,
            5437,
            567,
            3465241345,
            677,
            90,
            98765,
            4,
            324567897654321,
            3456,
            7754,
            32,
            4567,
            432,
            56789654321,
            4,
            5678906543,
            256,
            7895432,
            56789654,
            3256,
            78543,
        ];

        for i in ints {
            let c = compress(i);
            let (d, rest) = decompress(&c).unwrap();

            assert_eq!(d, i);
            assert_eq!(rest, Vec::new());
        }

        // now test if rest is parsed correctly

        for i in ints {
            let mut c = compress(i);
            c.push(1);
            c.push(2);
            c.push(3);
            c.push(4);
            let (d, rest) = decompress(&c).unwrap();

            assert_eq!(d, i);
            assert_eq!(rest, vec![1, 2, 3, 4]);
        }
    }

    #[test]
    fn list_compression() {
        let list = (0..100000).map(|n| n * 13).collect::<Vec<u64>>();
        let c = compress_list(&list);
        let d = decompress_list(&c).unwrap();

        assert_eq!(d, list);
    }

    #[test]
    fn compression_fuzzing() {
        let list = (0..100000u64).map(|n| n * 13);
        for elem in list {
            let c = compress(elem);
            let (d, rest) = decompress(&c).unwrap();

            assert_eq!(d, elem);
            assert!(rest.is_empty(), "no data should be remaining.");
        }
    }

    // --- failure cases ---

    #[test]
    fn decompress_empty() {
        assert!(decompress(&[]).is_err());
    }

    #[test]
    fn decompress_truncated_single() {
        // continuation bit set, no following byte
        assert!(decompress(&[0x80]).is_err());
    }

    #[test]
    fn decompress_truncated_multi() {
        // three bytes, all with continuation bit, no terminator
        assert!(decompress(&[0xFF, 0xFF, 0xFF]).is_err());
    }

    #[test]
    fn decompress_n_insufficient() {
        let buf = compress(1);
        assert!(decompress_n::<3>(&buf).is_err());
    }

    #[test]
    fn decompress_list_trailing_continuation() {
        let mut buf = compress_list(&[1u64, 2]);
        buf.push(0x80); // partial value at end
        assert!(decompress_list(&buf).is_err());
    }

    // --- byte-boundary values ---

    #[test]
    fn boundary_127() {
        assert_eq!(compress(127), [0x7F]);
        let (v, rest) = decompress(&[0x7F]).unwrap();
        assert_eq!(v, 127);
        assert!(rest.is_empty());
    }

    #[test]
    fn boundary_128() {
        assert_eq!(compress(128), [0x80, 0x01]);
        let (v, rest) = decompress(&[0x80, 0x01]).unwrap();
        assert_eq!(v, 128);
        assert!(rest.is_empty());
    }

    #[test]
    fn boundary_16383() {
        assert_eq!(compress(16383), [0xFF, 0x7F]);
        let (v, rest) = decompress(&[0xFF, 0x7F]).unwrap();
        assert_eq!(v, 16383);
        assert!(rest.is_empty());
    }

    #[test]
    fn boundary_16384() {
        assert_eq!(compress(16384), [0x80, 0x80, 0x01]);
        let (v, rest) = decompress(&[0x80, 0x80, 0x01]).unwrap();
        assert_eq!(v, 16384);
        assert!(rest.is_empty());
    }

    #[test]
    fn boundary_u64_max() {
        let mut expected = vec![0xFF; 9];
        expected.push(0x01);
        assert_eq!(compress(u64::MAX), expected);
        let (v, rest) = decompress(&expected).unwrap();
        assert_eq!(v, u64::MAX);
        assert!(rest.is_empty());
    }

    // --- consistency and trivial cases ---

    #[test]
    fn write_number_matches_compress() {
        for val in [0u64, 1, 127, 128, 16383, 16384, u64::MAX] {
            let mut buf = Vec::new();
            write_number(val, &mut buf).unwrap();
            assert_eq!(buf, compress(val), "mismatch for {val}");
        }
    }

    #[test]
    fn decompress_n_zero() {
        let data = &[1u8, 2, 3];
        let (arr, rest) = decompress_n::<0>(data).unwrap();
        assert_eq!(arr, []);
        assert_eq!(rest, data);
    }

    #[test]
    fn compress_list_empty() {
        assert_eq!(compress_list(&[]), Vec::<u8>::new());
    }

    #[test]
    fn decompress_list_empty() {
        assert_eq!(decompress_list(&[]).unwrap(), Vec::<u64>::new());
    }

    #[test]
    fn compress_list_zeros() {
        assert_eq!(compress_list(&[0u64, 0, 0]), [0u8, 0, 0]);
        assert_eq!(decompress_list(&[0u8, 0, 0]).unwrap(), [0u64, 0, 0]);
    }
}

/// Encodes `val` into `buffer` using variable-byte encoding.
pub fn write_number(mut val: u64, buffer: &mut impl Write) -> std::io::Result<()> {
    if val == 0 {
        buffer.write(&[0])?;
        return Ok(());
    }

    while val > 0 {
        // take the first 7 bytes of the value
        let mut byte = (val & 0b111_1111) as u8;

        // decrement value
        val >>= 7;

        // Set the `follow` byte,
        // if there remains information to be encoded
        if val > 0 {
            byte |= 0b1000_0000;
        }

        buffer.write(&[byte])?;
    }

    Ok(())
}

/// Encodes `val` as a variable-length byte vector.
pub fn compress(mut val: u64) -> Vec<u8> {
    if val == 0 {
        return vec![0];
    }

    let mut v = Vec::new();

    while val > 0 {
        // take the first 7 bytes of the value
        let mut byte = (val & 0b111_1111) as u8;

        // decrement value
        val >>= 7;

        // Set the `follow` byte,
        // if there remains information to be encoded
        if val > 0 {
            byte |= 0b1000_0000;
        }

        v.push(byte);
    }

    v.shrink_to_fit();
    v
}

/// Encodes a slice of integers into a contiguous byte buffer.
pub fn compress_list(vs: &[u64]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for v in vs {
        let c = compress(*v);
        buffer.extend(c);
    }

    buffer
}

/// Decodes one value from the front of `data`.
///
/// Returns the decoded integer and any remaining bytes. Returns an error
/// if the input ends before the value is complete.
pub fn decompress(data: &[u8]) -> Result<(u64, &[u8]), &str> {
    let mut val = 0u64;

    for i in 0..data.len() {
        let byte = data[i];
        let byte_index = i as u64 * 7;

        // update value
        {
            // cut of leading byte, if present
            let byte = (byte & 0b0111_1111) as u64;
            // decode proper position in value
            let byte = byte << byte_index;
            // assign to value
            val |= byte;
        }

        // continue?
        if byte & 0b1000_0000 != 0 {
            continue;
        }

        // end of value is reached, return

        let i = i + 1;
        let rest = &data[i..];
        return Ok((val, rest));
    }

    Err("end of input reached")
}

/// Decodes exactly `N` values from `data`, returning the remainder.
pub fn decompress_n<const N: usize>(mut data: &[u8]) -> Result<([u64; N], &[u8]), &str> {
    let mut out = [0; N];
    for entry in out.iter_mut() {
        let (val, rest) = decompress(data)?;
        *entry = val;
        data = rest;
    }

    Ok((out, data))
}

/// Decodes all values from `data` until the buffer is exhausted.
pub fn decompress_list(mut data: &[u8]) -> Result<Vec<u64>, &str> {
    let mut out = Vec::with_capacity(data.len());
    while !data.is_empty() {
        let (val, rest) = decompress(data)?;
        out.push(val);
        data = rest;
    }

    Ok(out)
}
