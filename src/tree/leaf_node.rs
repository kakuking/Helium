use std::io::Read;

use crate::{reader::{HeliumError, grid::CompressionFlags, helpers::Helper, vdb_constants::LEAF_NODE_SIZE}, tree::{coord::Coord, node_mask::NodeMask}};

#[derive(Debug)]
pub struct LeafNode {
    pub origin: Coord,
    pub value_mask: NodeMask,
    pub values: Vec<f32>,
}

impl LeafNode {
    pub fn read_topology<R: Read>(
        reader: &mut R,
        origin: Coord,
    ) -> Result<Self, HeliumError> {
        let value_mask =
            NodeMask::read(
                reader,
                LEAF_NODE_SIZE,
            )?;

        Ok(Self {
            origin,
            value_mask,
            values: Vec::new(),
        })
    }

    pub fn read_buffer<R: Read>(
        &mut self,
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<(), HeliumError> {
        if save_float_as_half {
            return Err(
                HeliumError::HalfFloatNotImplemented
            );
        }

        if compression.zip
            || compression.blosc
            || compression.active_mask
        {
            return Err(
                HeliumError::CompressedValuesNotImplemented {
                    flags: compression.raw,
                }
            );
        }

        let mut values =
            Vec::with_capacity(
                LEAF_NODE_SIZE,
            );

        for _ in 0..LEAF_NODE_SIZE {
            values.push(
                Helper::read_f32_le(reader)?
            );
        }

        self.values = values;

        Ok(())
    }
}