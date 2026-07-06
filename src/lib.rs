use std::io::Write;

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
