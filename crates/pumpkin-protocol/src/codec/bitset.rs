use std::io::{Error, ErrorKind, Read};

use crate::serial::PacketRead;

#[derive(Debug)]
pub struct Bitset<const N: usize> {
    pub bits: u128,
}

impl<const N: usize> Bitset<N> {
    pub fn get<T: Into<usize>>(&self, index: T) -> bool {
        let index: usize = index.into();
        assert!(index < N, "bit index {index} out of range for Bitset<{N}>");
        (self.bits & (1 << index)) != 0
    }

    pub fn set<T: Into<usize>>(&mut self, index: T, value: bool) {
        let index: usize = index.into();
        assert!(index < N, "bit index {index} out of range for Bitset<{N}>");
        if value {
            self.bits |= 1 << index;
        } else {
            self.bits &= !(1 << index);
        }
    }
}

impl<const N: usize> Default for Bitset<N> {
    fn default() -> Self {
        assert!(N <= 80,);
        Self { bits: 0 }
    }
}

impl<const N: usize> PacketRead for Bitset<N> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut bitset = Self::default();

        for i in 0..N.div_ceil(7) {
            let byte = u8::read(reader)?;
            bitset.bits |= (u128::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(bitset);
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "Bitset too large"))
    }
}

#[cfg(test)]
mod tests {
    use super::Bitset;

    #[test]
    fn last_valid_index_is_n_minus_one() {
        let mut bits = Bitset::<66>::default();
        bits.set(65usize, true);
        assert!(bits.get(65usize));
        assert!(!bits.get(64usize));
    }

    #[test]
    #[should_panic(expected = "out of range for Bitset")]
    fn index_n_is_out_of_range() {
        let bits = Bitset::<66>::default();
        let _ = bits.get(66usize);
    }
}
