use std::io::{Read, Seek, SeekFrom};

use crate::reader::{
    HeliumError, 
    grid_descriptor::GridDescriptor, 
    helpers::Helper, 
    metadata::Metadata, 
    transform::Transform
};

#[derive(Debug)]
pub struct Grid {
    pub descriptor: GridDescriptor,

    pub compression: CompressionFlags,
    pub metadata: Vec<Metadata>,
    pub transform: Transform,

    pub raw_topology: Vec<u8>,

    pub raw_buffers: Vec<u8>,
}

impl Grid {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        descriptor: &GridDescriptor,
        grid_index: usize,
        file_length: u64,
    ) -> Result<Grid, HeliumError> {
        let grid_position = Helper::checked_offset(descriptor.grid_position, "grid_position")?;
        let block_position = Helper::checked_offset(descriptor.block_position, "block_position")?;
        let end_position = Helper::checked_offset(descriptor.end_position, "end_position")?;

        if end_position > file_length {
            return Err(HeliumError::GridOffsetPastEnd{ 
                grid_index, 
                end_position, 
                file_length
            });
        }

        reader.seek(SeekFrom::Start(grid_position))?;

        let compression_raw = Helper::read_u32_le(reader)?;
        let compression = CompressionFlags::from_raw(compression_raw);

        if compression.unknown_bits() != 0 {
            return Err(HeliumError::UnknownCompressionFlags {
                grid_index,
                flags: compression_raw,
            });
        }

        let metadata = Metadata::read(reader)?;
        let transform = Transform::read(reader)?;

        let topology_start = reader.stream_position()?;

        if topology_start > block_position {
            return Err(HeliumError::GridSectionOverlap{
                grid_index, 
                topology_start, 
                block_position
            });
        }

        let is_instance = !descriptor.instance_parent_name.is_empty();
        
        let raw_topology = if is_instance {
            Vec::new()
        } else {
            Helper::read_byte_range(
                reader, 
                topology_start, 
                block_position
            )?
        };

        let raw_buffers = if is_instance {

            Vec::new()
        } else {
            Helper::read_byte_range(
                reader, 
                block_position, 
                end_position
            )?
        };

        Ok(Grid {
            descriptor: descriptor.clone(),
            compression,
            metadata,
            transform,
            raw_topology,
            raw_buffers
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompressionFlags {
    pub raw: u32,
    pub zip: bool,
    pub active_mask: bool,
    pub blosc: bool,
}

impl CompressionFlags {
    pub const ZIP: u32 = 0x1;
    pub const ACTIVE_MASK: u32 = 0x2;
    pub const BLOSC: u32 = 0x4;

    pub fn from_raw(raw: u32) -> Self {
        Self {
            raw,
            zip: raw & Self::ZIP != 0,
            active_mask: raw & Self::ACTIVE_MASK != 0,
            blosc: raw & Self::BLOSC != 0,
        }
    }

    pub fn unknown_bits(self) -> u32 {
        self.raw & !(Self::ZIP | Self::ACTIVE_MASK | Self::BLOSC)
    }
}
