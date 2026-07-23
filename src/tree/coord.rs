use std::io::Read;

use crate::reader::{HeliumError, helpers::Helper};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Coord {
    pub fn read<R: Read>(
        reader: &mut R,
    ) -> Result<Coord, HeliumError> {
        Ok(Coord {
            x: Helper::read_i32_le(reader)?,
            y: Helper::read_i32_le(reader)?,
            z: Helper::read_i32_le(reader)?,
        })
    }
}