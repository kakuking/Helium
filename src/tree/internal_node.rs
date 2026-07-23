use std::io::Read;

use crate::{reader::{HeliumError, grid::CompressionFlags, helpers::Helper, vdb_constants::{LOWER_NODE_LOG2_DIM, LOWER_NODE_SIZE, UPPER_NODE_LOG2_DIM, UPPER_NODE_SIZE}}, tree::{coord::Coord, leaf_node::LeafNode, node_mask::NodeMask}};

#[derive(Debug)]
pub struct UpperInternalNode {
    pub origin: Coord,

    pub child_mask: NodeMask,
    pub value_mask: NodeMask,

    pub values: Vec<f32>,
    pub children: Vec<LowerInternalNode>,
}

#[derive(Debug)]
pub struct LowerInternalNode {
    pub origin: Coord,

    pub child_mask: NodeMask,
    pub value_mask: NodeMask,

    pub values: Vec<f32>,
    pub children: Vec<LeafNode>,
}

fn offset_to_local(
    offset: usize,
    log2_dim: usize,
) -> [usize; 3] {
    let dimension = 1usize << log2_dim;
    let mask = dimension - 1;

    let x = offset >> (2 * log2_dim);
    let y = (offset >> log2_dim) & mask;
    let z = offset & mask;

    [x, y, z]
}

fn upper_child_origin(
    node_origin: Coord,
    slot: usize,
) -> Coord {
    let [x, y, z] =
        offset_to_local(
            slot,
            UPPER_NODE_LOG2_DIM,
        );

    const LOWER_VOXEL_EXTENT: i32 =
        16 * 8;

    Coord {
        x: node_origin.x
            + x as i32 * LOWER_VOXEL_EXTENT,

        y: node_origin.y
            + y as i32 * LOWER_VOXEL_EXTENT,

        z: node_origin.z
            + z as i32 * LOWER_VOXEL_EXTENT,
    }
}

fn lower_child_origin(
    node_origin: Coord,
    slot: usize,
) -> Coord {
    let [x, y, z] =
        offset_to_local(
            slot,
            LOWER_NODE_LOG2_DIM,
        );

    const LEAF_VOXEL_EXTENT: i32 = 8;

    Coord {
        x: node_origin.x
            + x as i32 * LEAF_VOXEL_EXTENT,

        y: node_origin.y
            + y as i32 * LEAF_VOXEL_EXTENT,

        z: node_origin.z
            + z as i32 * LEAF_VOXEL_EXTENT,
    }
}

fn read_internal_values<R: Read>(
    reader: &mut R,
    entry_count: usize,
    child_mask: &NodeMask,
    compression: CompressionFlags,
    save_float_as_half: bool,
) -> Result<Vec<f32>, HeliumError> {
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

    let value_count =
        entry_count - child_mask.count_on();

    let mut values =
        Vec::with_capacity(value_count);

    for _ in 0..value_count {
        values.push(
            Helper::read_f32_le(reader)?
        );
    }

    Ok(values)
}

impl UpperInternalNode {
    pub fn read_topology<R: Read>(
        reader: &mut R,
        origin: Coord,
        background: f32,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<Self, HeliumError> {
        let child_mask =
            NodeMask::read(
                reader,
                UPPER_NODE_SIZE,
            )?;

        let value_mask =
            NodeMask::read(
                reader,
                UPPER_NODE_SIZE,
            )?;

        let values =
            read_internal_values(
                reader,
                UPPER_NODE_SIZE,
                &child_mask,
                compression,
                save_float_as_half,
            )?;

        let mut children =
            Vec::with_capacity(
                child_mask.count_on(),
            );

        for slot in 0..UPPER_NODE_SIZE {
            if !child_mask.is_on(slot) {
                continue;
            }

            let child_origin =
                upper_child_origin(
                    origin,
                    slot,
                );

            let child =
                LowerInternalNode::read_topology(
                    reader,
                    child_origin,
                    background,
                    compression,
                    save_float_as_half,
                )?;

            children.push(child);
        }

        Ok(Self {
            origin,
            child_mask,
            value_mask,
            values,
            children,
        })
    }

    pub fn read_buffers<R: Read>(
        &mut self,
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<(), HeliumError> {
        for child in &mut self.children {
            child.read_buffers(
                reader,
                compression,
                save_float_as_half,
            )?;
        }

        Ok(())
    }
}

impl LowerInternalNode {
    pub fn read_topology<R: Read>(
        reader: &mut R,
        origin: Coord,
        _background: f32,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<Self, HeliumError> {
        let child_mask =
            NodeMask::read(
                reader,
                LOWER_NODE_SIZE,
            )?;

        let value_mask =
            NodeMask::read(
                reader,
                LOWER_NODE_SIZE,
            )?;

        let values =
            read_internal_values(
                reader,
                LOWER_NODE_SIZE,
                &child_mask,
                compression,
                save_float_as_half,
            )?;

        let mut children =
            Vec::with_capacity(
                child_mask.count_on(),
            );

        for slot in 0..LOWER_NODE_SIZE {
            if !child_mask.is_on(slot) {
                continue;
            }

            let child_origin =
                lower_child_origin(
                    origin,
                    slot,
                );

            let child =
                LeafNode::read_topology(
                    reader,
                    child_origin,
                )?;

            children.push(child);
        }

        Ok(Self {
            origin,
            child_mask,
            value_mask,
            values,
            children,
        })
    }

    pub fn read_buffers<R: Read>(
        &mut self,
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<(), HeliumError> {
        for child in &mut self.children {
            child.read_buffer(
                reader,
                compression,
                save_float_as_half,
            )?;
        }

        Ok(())
    }
}