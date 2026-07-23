use std::io::Read;

use crate::reader::{HeliumError, helpers::Helper};

#[derive(Debug, Clone)]
pub struct GridDescriptor {
    pub unique_name: String,
    pub grid_type: String,
    pub instance_parent_name: String,

    pub grid_position: i64,
    pub block_position: i64,
    pub end_position: i64,

    pub save_float_as_half: bool,
}

impl GridDescriptor {
    pub fn read<R: Read>(reader: &mut R) -> Result<GridDescriptor, HeliumError> {
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

    pub fn validate(
        &self,
        grid_index: usize,
        file_length: u64,
    ) -> Result<(), HeliumError> {
        if self.grid_position < 0
            || self.block_position < 0
            || self.end_position < 0
        {
            return Err(HeliumError::NegativeGridOffsets {
                grid_index,
                grid_position: self.grid_position,
                block_position: self.block_position,
                end_position: self.end_position,
            });
        }
        
        if self.grid_position > self.block_position {
            return Err(HeliumError::InvalidGridOffsets{
                grid_index, 
                grid_position: self.grid_position, 
                block_position: self.block_position, 
                end_position: self.end_position 
            });
        }

        if self.block_position > self.end_position {
            return Err(HeliumError::InvalidGridOffsets{
                grid_index, 
                grid_position: self.grid_position, 
                block_position: self.block_position, 
                end_position: self.end_position 
            });
        }

        let end_position = self.end_position as u64;

        if end_position > file_length {
            return Err(HeliumError::GridOffsetPastEnd {
                grid_index,
                end_position,
                file_length,
            });
        }

        Ok(())
    }
}