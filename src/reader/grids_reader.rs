use std::io::{Read, Seek, SeekFrom};

use crate::reader::{
    HeliumError, 
    grid::{
        CompressionFlags, 
        Grid, 
        GridDescriptor
    }, 
    header::Header, 
    helpers::Helper, 
    metadata::Metadata, 
    vdb_constants::MAX_GRID_COUNT
};

#[derive(Debug)]
pub struct Grids {
    pub count: usize,

    pub descriptors: Vec<GridDescriptor>,
    pub grids: Vec<Grid>,
}

impl Grids {
    pub fn new() -> Self {
        Self {
            count: 0,

            descriptors: Vec::new(),
            grids: Vec::new(),
        }
    }

    pub fn read<R: Read + Seek>(
        reader: &mut R,
        header: &Header,
        file_length: u64
    ) -> Result<Self, HeliumError> {
        let count = Self::read_grid_count(reader)?;

        let descriptors = Self::read_grid_descriptors(
            reader, 
            header,
            count, 
            file_length
        )?;

        let grids = Self::read_grids(
            reader, 
            &descriptors, 
            file_length
        )?;

        Ok(Self {
            count,

            descriptors,
            grids
        })
    }

    fn read_grid_count<R: Read>(reader: &mut R) -> Result<usize, HeliumError> {
        let grid_count = Helper::read_i32_le(reader)?;

        if !(0..=MAX_GRID_COUNT).contains(&grid_count) {
            return Err(HeliumError::InvalidGridCount(grid_count));
        }

        Ok(grid_count as usize)
    }

    fn read_grid_descriptors<R: Read + Seek>(
        reader: &mut R,
        header: &Header,
        grid_count: usize,
        file_length: u64
    ) -> Result<Vec<GridDescriptor>, HeliumError> {
        if !header.has_grid_offsets {
            return Err(HeliumError::GridOffsetsRequired);
        }

        let mut grid_descriptors: Vec<GridDescriptor> = Vec::new();
        grid_descriptors.reserve(grid_count);

        for grid_index in 0..grid_count {
            let descriptor = Self::read_grid_descriptor(reader)?;

            Self::validate_grid_descriptor(
                grid_index, 
                &descriptor, 
                file_length
            )?;

            let next_descriptor_position = descriptor.end_position;

            grid_descriptors.push(descriptor);

            reader.seek(SeekFrom::Start(next_descriptor_position as u64))?;
        }

        Ok(grid_descriptors)
    }

    fn read_grid_descriptor<R: Read>(reader: &mut R) -> Result<GridDescriptor, HeliumError> {
        let unique_name = Helper::read_string(reader)?;
        let mut grid_type = Helper::read_string(reader)?;
        let instance_parent_name = Helper::read_string(reader)?;

        let grid_position = Helper::read_i64_le(reader)?;
        let block_position = Helper::read_i64_le(reader)?;
        let end_position = Helper::read_i64_le(reader)?;

        const HALF_FLOAT_SUFFIX: &str = "_HalfFloat";

        let save_float_as_half = grid_type.ends_with(HALF_FLOAT_SUFFIX);

        if save_float_as_half {
            grid_type.truncate(
                grid_type.len() - HALF_FLOAT_SUFFIX.len()
            );
        }

        Ok(GridDescriptor{
            unique_name,
            grid_type,
            instance_parent_name,
            grid_position,
            block_position,
            end_position,
            save_float_as_half
        })
    }

    fn validate_grid_descriptor(
        grid_index: usize,
        descriptor: &GridDescriptor,
        file_length: u64,
    ) -> Result<(), HeliumError> {
        if descriptor.grid_position < 0
            || descriptor.block_position < 0
            || descriptor.end_position < 0
        {
            return Err(HeliumError::NegativeGridOffsets {
                grid_index,
                grid_position: descriptor.grid_position,
                block_position: descriptor.block_position,
                end_position: descriptor.end_position,
            });
        }
        
        if descriptor.grid_position > descriptor.block_position {
            return Err(HeliumError::InvalidGridOffsets{
                grid_index, 
                grid_position: descriptor.grid_position, 
                block_position: descriptor.block_position, 
                end_position: descriptor.end_position 
            });
        }

        if descriptor.block_position > descriptor.end_position {
            return Err(HeliumError::InvalidGridOffsets{
                grid_index, 
                grid_position: descriptor.grid_position, 
                block_position: descriptor.block_position, 
                end_position: descriptor.end_position 
            });
        }

        let end_position = descriptor.end_position as u64;

        if end_position > file_length {
            return Err(HeliumError::GridOffsetPastEnd {
                grid_index,
                end_position,
                file_length,
            });
        }

        Ok(())
    }

    fn read_grids<R: Read + Seek>(
        reader: &mut R,
        descriptors: &[GridDescriptor],
        file_length: u64
    ) -> Result<Vec<Grid>, HeliumError> {
        let mut grids = Vec::with_capacity(descriptors.len());

        for (grid_index, descriptor) in descriptors.iter().enumerate() {
            println!(
                "Reading grid {} ({:?}) at {}..{}",
                grid_index,
                descriptor.unique_name,
                descriptor.grid_position,
                descriptor.end_position,
            );
            
            let grid = Self::read_grid(
                reader, 
                descriptor,
                grid_index,
                file_length,
            )?;

            grids.push(grid);
        }

        Ok(grids)
    }

    fn read_grid<R: Read + Seek>(
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
        let transform = Helper::read_transform(reader)?;

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

