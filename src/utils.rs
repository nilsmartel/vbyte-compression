/// Trait for types that can be encoded as a sequence of `u64` values and decoded back.
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

pub struct CompressedList<T: VbyteEncode> {
    pub compressed_data: Vec<u8>,
    _ty: PhantomData<T>,
}
