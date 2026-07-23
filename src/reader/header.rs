use std::io::Read;

use crate::reader::{
    HeliumError, 
    helpers::Helper, 
    vdb_constants::{
        MAX_SUPPORTED_VERSION, 
        MIN_SUPPORTED_VERSION, 
        OPENVDB_MAGIC
    }
};

#[derive(Debug)]
pub struct Header {
    pub file_version: u32,
    pub library_major: u32,
    pub library_minor: u32,
    pub has_grid_offsets: bool,
    pub uuid: String,
}

impl Header {
    pub fn new() -> Self {
        Header {
            file_version: 0,
            library_major: 0,
            library_minor: 0,
            has_grid_offsets: false,
            uuid: "".into()
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, HeliumError> {
        let magic = Helper::read_u64_le(reader)?;

        if magic != OPENVDB_MAGIC {
            return Err(
                HeliumError::InvalidMagic{
                    expected: OPENVDB_MAGIC,
                    found: magic,
                }
            );
        }

        let file_version = Helper::read_u32_le(reader)?;

        if !(MIN_SUPPORTED_VERSION..=MAX_SUPPORTED_VERSION).contains(&file_version) {
            return Err(HeliumError::UnsupportedVersion(file_version));
        }

        let library_major = Helper::read_u32_le(reader)?;
        let library_minor = Helper::read_u32_le(reader)?;

        let has_grid_offsets_raw = Helper::read_u8(reader)?;
        let has_grid_offsets = match has_grid_offsets_raw {
            0 => false,
            1 => true,
            value => {
                return Err(HeliumError::InvalidBoolean {
                    field: "has_grid_offsets", 
                    value
                });
            }
        };

        let mut uuid_bytes = [0u8; 36];
        reader.read_exact(&mut uuid_bytes)?;

        let uuid = std::str::from_utf8(&uuid_bytes)
            .map_err(|_| HeliumError::InvalidUuidEncoding)?
            .to_owned();

        Ok(Header{ 
            file_version, 
            library_major, 
            library_minor, 
            has_grid_offsets, 
            uuid
        })
    }
}