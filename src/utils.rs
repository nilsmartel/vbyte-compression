use std::marker::PhantomData;

/// Trait for types that can be encoded as a sequence of `u64` values and decoded back.
///
/// Implement `encode` and `decode`; `compress` is provided for free.
///
/// # Example
///
/// ```rust
/// # use vbyte::utils::VbyteEncode;
/// struct Point { x: u32, y: u32 }
///
/// impl VbyteEncode for Point {
///     fn encode(&self, out: &mut Vec<u64>) {
///         self.x.encode(out);
///         self.y.encode(out);
///     }
///     fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
///         let (x, fields) = u32::decode(fields)?;
///         let (y, fields) = u32::decode(fields)?;
///         Ok((Point { x, y }, fields))
///     }
/// }
///
/// let points = vec![Point { x: 1, y: 2 }, Point { x: 314, y: 159 }];
/// let compressed = Point::compress(&points);
/// let restored = compressed.decompress().unwrap();
/// ```
pub trait VbyteEncode: Sized {
    /// Append the u64 representation of `self` to `out`.
    fn encode(&self, out: &mut Vec<u64>);

    /// Decode one value from the front of `fields`, returning the remainder.
    ///
    /// Returns `Err` if the input is too short or a value is out of range.
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str>;

    /// Compress a slice of values into a [`CompressedList`].
    fn compress(items: &[Self]) -> CompressedList<Self> {
        let mut raw: Vec<u64> = Vec::new();
        for item in items {
            item.encode(&mut raw);
        }
        CompressedList {
            compressed_data: crate::compress_list(&raw),
            _ty: PhantomData,
        }
    }
}

/// A compressed sequence of values of type `T`.
///
/// Obtained via [`VbyteEncode::compress`] or the free [`compress`] function.
pub struct CompressedList<T: VbyteEncode> {
    pub compressed_data: Vec<u8>,
    _ty: PhantomData<T>,
}

impl<T: VbyteEncode> CompressedList<T> {
    /// Returns the raw compressed bytes.
    pub fn get_compressed_bytes(&self) -> &[u8] {
        &self.compressed_data
    }

    /// Decodes all values from the compressed buffer.
    pub fn decompress(&self) -> Result<Vec<T>, &'static str> {
        let raw = crate::decompress_list(&self.compressed_data)
            .map_err(|_| "byte buffer corrupt or truncated")?;
        let mut fields: &[u64] = &raw;
        let mut out = Vec::new();
        while !fields.is_empty() {
            let (val, rest) = T::decode(fields)?;
            out.push(val);
            fields = rest;
        }
        Ok(out)
    }
}

// Blank implementation for primitive types.

macro_rules! impl_via_u64 {
    ($t:ty) => {
        impl VbyteEncode for $t {
            fn encode(&self, out: &mut Vec<u64>) {
                out.push(*self as u64);
            }
            fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
                let (&v, rest) = fields.split_first().ok_or("not enough data")?;
                let val = <$t>::try_from(v).map_err(|_| "value out of range")?;
                Ok((val, rest))
            }
        }
    };
}

impl VbyteEncode for u64 {
    fn encode(&self, out: &mut Vec<u64>) {
        out.push(*self);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        fields
            .split_first()
            .map(|(&v, rest)| (v, rest))
            .ok_or("not enough data")
    }
}

impl VbyteEncode for i64 {
    fn encode(&self, out: &mut Vec<u64>) {
        out.push(*self as u64);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        fields
            .split_first()
            .map(|(&v, rest)| (v as i64, rest))
            .ok_or("not enough data")
    }
}

impl_via_u64!(u32);
impl_via_u64!(u16);
impl_via_u64!(u8);
impl_via_u64!(i32);
impl_via_u64!(i16);
impl_via_u64!(i8);

impl VbyteEncode for bool {
    fn encode(&self, out: &mut Vec<u64>) {
        out.push(*self as u64);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        let (&v, rest) = fields.split_first().ok_or("not enough data")?;
        match v {
            0 => Ok((false, rest)),
            1 => Ok((true, rest)),
            _ => Err("value out of range for bool"),
        }
    }
}

impl VbyteEncode for f64 {
    fn encode(&self, out: &mut Vec<u64>) {
        let data: u64 = f64::to_bits(*self) ;
        out.push(data);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        fields
            .split_first()
            .map(|(&v, rest)| (f64::from_bits(v) , rest))
            .ok_or("not enough data")
    }
}

impl VbyteEncode for f32 {
    fn encode(&self, out: &mut Vec<u64>) {
        let data: u64 = f32::to_bits(*self) as u64 ;
        out.push(data);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        fields
            .split_first()
            .map(|(&v, rest)| (f32::from_bits(v as u32), rest))
            .ok_or("not enough data")
    }
}

// Blank implementation for tuples

impl<A: VbyteEncode, B: VbyteEncode> VbyteEncode for (A, B) {
    fn encode(&self, out: &mut Vec<u64>) {
        self.0.encode(out);
        self.1.encode(out);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        let (a, fields) = A::decode(fields)?;
        let (b, fields) = B::decode(fields)?;
        Ok(((a, b), fields))
    }
}

impl<A: VbyteEncode, B: VbyteEncode, C: VbyteEncode> VbyteEncode for (A, B, C) {
    fn encode(&self, out: &mut Vec<u64>) {
        self.0.encode(out);
        self.1.encode(out);
        self.2.encode(out);
    }
    fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
        let (a, fields) = A::decode(fields)?;
        let (b, fields) = B::decode(fields)?;
        let (c, fields) = C::decode(fields)?;
        Ok(((a, b, c), fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MyPoint {
        x: u32,
        y: u32,
    }

    impl VbyteEncode for MyPoint {
        fn encode(&self, out: &mut Vec<u64>) {
            self.x.encode(out);
            self.y.encode(out);
        }
        fn decode(fields: &[u64]) -> Result<(Self, &[u64]), &'static str> {
            let (x, fields) = u32::decode(fields)?;
            let (y, fields) = u32::decode(fields)?;
            Ok((MyPoint { x, y }, fields))
        }
    }

    #[test]
    fn roundtrip_mypoint() {
        let points = vec![
            MyPoint { x: 0, y: 0 },
            MyPoint { x: 100, y: 200 },
            MyPoint { x: u32::MAX, y: u32::MAX },
        ];
        let compressed = MyPoint::compress(&points);
        let restored = compressed.decompress().unwrap();
        assert_eq!(restored, points);
    }

    #[test]
    fn roundtrip_mypoint_bytes_accessible() {
        let points = vec![MyPoint { x: 1, y: 2 }];
        let compressed = MyPoint::compress(&points);
        assert!(!compressed.get_compressed_bytes().is_empty());
    }

    #[test]
    fn roundtrip_u8() {
        let original: Vec<u8> = vec![0, 1, 127, 255];
        let c = compress(&original);
        let d: Vec<u8> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn roundtrip_u32() {
        let original: Vec<u32> = vec![0, 1, 128, u32::MAX];
        let c = compress(&original);
        let d: Vec<u32> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn roundtrip_u64() {
        let original: Vec<u64> = vec![0, u64::MAX, 12345678];
        let c = compress(&original);
        let d: Vec<u64> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn roundtrip_bool() {
        let original = vec![true, false, false, true];
        let c = compress(&original);
        let d: Vec<bool> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn roundtrip_tuple() {
        let original: Vec<(u32, u32)> = vec![(0, 1), (128, 255), (u32::MAX, 0)];
        let c = compress(&original);
        let d: Vec<(u32, u32)> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn roundtrip_triple() {
        let original: Vec<(u8, u16, u32)> = vec![(0, 1, 2), (255, 65535, u32::MAX)];
        let c = compress(&original);
        let d: Vec<(u8, u16, u32)> = c.decompress().unwrap();
        assert_eq!(d, original);
    }

    #[test]
    fn conversion_failure() {
        // 300 does not fit in u8
        let c: CompressedList<u8> = CompressedList {
            compressed_data: crate::compress_list(&[300u64]),
            _ty: PhantomData,
        };
        assert!(c.decompress().is_err());
    }

    #[test]
    fn invalid_data() {
        let c: CompressedList<u64> = CompressedList {
            compressed_data: vec![0x80], // truncated continuation byte
            _ty: PhantomData,
        };
        assert!(c.decompress().is_err());
    }
}
