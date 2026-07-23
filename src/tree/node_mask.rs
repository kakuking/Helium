use std::io::Read;

use crate::reader::{
    HeliumError,
    helpers::Helper,
};

#[derive(Debug)]
pub struct NodeMask {
    pub bit_count: usize,
    pub words: Vec<u64>,
}

impl NodeMask {
    pub fn read<R: Read>(
        reader: &mut R,
        bit_count: usize,
    ) -> Result<Self, HeliumError> {
        let word_count = bit_count.div_ceil(64);

        let mut words = Vec::with_capacity(
            word_count,
        );

        for _ in 0..word_count {
            words.push(
                Helper::read_u64_le(reader)?
            );
        }

        Ok(Self {
            bit_count,
            words,
        })
    }

    pub fn is_on(&self, index: usize) -> bool {
        debug_assert!(index < self.bit_count);

        let word_index = index / 64;
        let bit_index = index % 64;

        self.words[word_index]
            & (1_u64 << bit_index)
            != 0
    }

    pub fn count_on(&self) -> usize {
        self.words
            .iter()
            .map(|word| {
                word.count_ones() as usize
            })
            .sum()
    }
}