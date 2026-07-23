use std::io::{Read, Seek};

use crate::{reader::{HeliumError, grid::CompressionFlags, helpers::Helper}, tree::root_node::RootNode};

#[derive(Debug)]
pub struct Tree {
    pub root: RootNode
}

impl Tree {
    pub fn read_topology<R: Read + Seek>(
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<Self, HeliumError> {
        let buffer_count = Helper::read_i32_le(reader)?;

        if buffer_count <= 0 {
            return Err(
                HeliumError::InvalidTreeBufferCount(
                    buffer_count,
                ),
            );
        }

        let start = reader.stream_position()?;
        println!("Tree topology starts at {start}");

        let root = RootNode::read_topology(
            reader,
            compression,
            save_float_as_half,
        )?;

        let end = reader.stream_position()?;
        println!(
            "Tree topology consumed {} bytes: {}..{}",
            end - start,
            start,
            end,
        );

        Ok(Self {
            root,
        })
    }

    pub fn read_buffers<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        compression: CompressionFlags,
        save_float_as_half: bool,
    ) -> Result<(), HeliumError> {
        self.root.read_buffers(
            reader,
            compression,
            save_float_as_half,
        )
    }
}