use std::io::{Read, Seek, SeekFrom};

use crate::reader::{
    HeliumError, 
    vdb_constants::MAX_STRING_LENGTH
};

pub struct Helper {}

impl Helper {
    pub fn read_string<R: Read>(
        reader: &mut R,
    ) -> Result<String, HeliumError> {
        let length = Self::read_u32_le(reader)? as usize;

        if length > MAX_STRING_LENGTH {
            return Err(HeliumError::StringTooLarge(length));
        }

        let mut bytes = vec![0_u8; length];
        reader.read_exact(&mut bytes)?;

        String::from_utf8(bytes)
            .map_err(HeliumError::InvalidStringEncoding)
    }

    pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8, HeliumError> {
        let mut bytes = [0_u8; 1];
        reader.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }
    
    pub fn read_i16_le<R: Read>(reader: &mut R) -> Result<i16, HeliumError> {
        let mut bytes = [0_u8; 2];
        reader.read_exact(&mut bytes)?;
        Ok(i16::from_le_bytes(bytes))
    }

    pub fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, HeliumError> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_i32_le<R: Read>(reader: &mut R) -> Result<i32, HeliumError> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(i32::from_le_bytes(bytes))
    }

    pub fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn read_i64_le<R: Read>(reader: &mut R) -> Result<i64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(i64::from_le_bytes(bytes))
    }

    pub fn read_byte_range<R: Read + Seek>(
        reader: &mut R,
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>, HeliumError> {
        let length = end
            .checked_sub(start)
            .ok_or(HeliumError::InvalidByteRange { start, end })?;

        let length = usize::try_from(length)
            .map_err(|_| HeliumError::ByteRangeTooLarge { start, end })?;

        reader.seek(SeekFrom::Start(start))?;

        let mut bytes = vec![0_u8; length];
        reader.read_exact(&mut bytes)?;

        Ok(bytes)
    }

    pub fn read_f64_le<R: Read>(
        reader: &mut R,
    ) -> Result<f64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(f64::from_le_bytes(bytes))
    }

    pub fn read_vec3d<R: Read>(
        reader: &mut R,
    ) -> Result<[f64; 3], HeliumError> {
        Ok([
            Self::read_f64_le(reader)?,
            Self::read_f64_le(reader)?,
            Self::read_f64_le(reader)?,
        ])
    }

    pub fn checked_offset(
        offset: i64,
        field: &'static str,
    ) -> Result<u64, HeliumError> {
        u64::try_from(offset).map_err(|_| {
            HeliumError::NegativeStreamPosition {
                field,
                value: offset,
            }
        })
    }

    pub fn read_mat4d<R: Read>(
        reader: &mut R,
    ) -> Result<[[f64; 4]; 4], HeliumError> {
        let mut matrix = [[0.0_f64; 4]; 4];

        for row in &mut matrix {
            for value in row {
                *value = Self::read_f64_le(reader)?;
            }
        }

        Ok(matrix)
    }
}