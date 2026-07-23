use std::io::Read;

use crate::reader::{
    HeliumError, 
    helpers::Helper, 
    vdb_constants::{
        MAX_METADATA_COUNT, 
        MAX_STRING_LENGTH
    }
};

#[derive(Debug)]
pub struct Metadata {
    pub name: String,
    pub type_name: String,

    pub payload: Vec<u8>,
}

impl Metadata {
    pub fn read<R: Read>(reader: &mut R) -> Result<Vec<Self>, HeliumError> {
        let metadata_count = Helper::read_u32_le(reader)?;

        if metadata_count > MAX_METADATA_COUNT {
            return Err(
                HeliumError::InvalidMetadataCount(metadata_count)
            );
        }

        let mut metadata = Vec::with_capacity(metadata_count as usize);

        for _ in 0..metadata_count {
            let name = Helper::read_string(reader)?;
            let type_name = Helper::read_string(reader)?;

            let payload_size = Helper::read_u32_le(reader)? as usize;

            if payload_size > MAX_STRING_LENGTH {
                return Err(HeliumError::MetadataPayloadTooLarge{
                    name, 
                    size: payload_size
                });
            }

            let mut payload = vec![0u8; payload_size];
            reader.read_exact(&mut payload)?;

            metadata.push(
                Metadata {
                    name,
                    type_name,
                    payload
                }
            );
        }

        Ok(
            metadata
        )
    }
}