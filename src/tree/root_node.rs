use std::io::Read;

use crate::{reader::{HeliumError, grid::CompressionFlags, helpers::Helper}, tree::{coord::Coord, internal_node::UpperInternalNode}};

#[derive(Debug)]
pub struct RootNode {
    pub background: f32,
    pub tiles: Vec<RootTile>,
    pub children: Vec<RootChild>,
}

#[derive(Debug)]
pub struct RootTile {
    pub origin: Coord,
    pub value: f32,
    pub active: bool,
}

#[derive(Debug)]
pub struct RootChild {
    pub origin: Coord,
    pub node: UpperInternalNode,
}

impl RootNode {
    pub fn read_topology<R: Read>(
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<Self, HeliumError> {
        if save_float_as_half {
            return Err(
                HeliumError::HalfFloatNotImplemented
            );
        }

        let background =
            Helper::read_f32_le(reader)?;

        let tile_count =
            Helper::read_u32_le(reader)? as usize;

        let child_count =
            Helper::read_u32_le(reader)? as usize;

        let mut tiles =
            Vec::with_capacity(tile_count);

        for _ in 0..tile_count {
            let origin =
                Coord::read(reader)?;

            let value =
                Helper::read_f32_le(reader)?;

            let active =
                Helper::read_bool(
                    reader,
                    "root_tile_active",
                )?;

            tiles.push(RootTile {
                origin,
                value,
                active,
            });
        }

        let mut children =
            Vec::with_capacity(child_count);

        for _ in 0..child_count {
            let origin =
                Coord::read(reader)?;

            let node =
                UpperInternalNode::read_topology(
                    reader,
                    origin,
                    background,
                    compression,
                    save_float_as_half,
                )?;

            children.push(RootChild {
                origin,
                node,
            });
        }

        Ok(Self {
            background,
            tiles,
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
            child.node.read_buffers(
                reader,
                compression,
                save_float_as_half,
            )?;
        }

        Ok(())
    }
}