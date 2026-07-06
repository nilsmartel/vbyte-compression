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
