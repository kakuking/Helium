use std::io::{Read, Seek, SeekFrom};

use crate::reader::{
    HeliumError, 
    grid::{
        Grid, 
    }, 
    grid_descriptor::GridDescriptor, 
    header::Header, 
    helpers::Helper, 
    vdb_constants::MAX_GRID_COUNT
};

#[derive(Debug)]
pub struct Grids {
    pub descriptors: Vec<GridDescriptor>,
    // pub grids: Vec<Grid>,
}

impl Grids {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
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

        // let grids = Self::read_grids(
        //     reader, 
        //     &descriptors, 
        //     file_length
        // )?;

        Ok(Self {
            descriptors,
        })
    }

    pub fn descriptor(
        &self,
        name: &str
    ) -> Option<&GridDescriptor> {
        self.descriptors
            .iter()
            .find(|d| d.unique_name == name)
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
            let descriptor = GridDescriptor::read(reader)?;

            descriptor.validate(
                grid_index, 
                file_length
            )?;

            let next_descriptor_position = descriptor.end_position;

            grid_descriptors.push(descriptor);

            reader.seek(SeekFrom::Start(next_descriptor_position as u64))?;
        }

        Ok(grid_descriptors)
    }

    pub fn read_grids<R: Read + Seek>(
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
            
            let grid = Grid::read(
                reader, 
                descriptor,
                grid_index,
                file_length,
            )?;

            grids.push(grid);
        }

        Ok(grids)
    }

    
}

