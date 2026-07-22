use std::{
    fmt::Debug, fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::Path
};

use crate::reader::{HeliumError, vdb_structs::{CompressionFlags, GridDescriptor, MAX_GRID_COUNT, MAX_METADATA_COUNT, MAX_STRING_LENGTH, MAX_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION, OPENVDB_MAGIC, VdbGrid, VdbHeader, VdbMetadata, VdbTransform}};

#[derive(Debug)]
pub struct HeliumReader {
    file_path: String,

    header: VdbHeader,
    file_metadata: Vec<VdbMetadata>,
    grid_descriptors: Vec<GridDescriptor>,
    grids: Vec<VdbGrid>,
}


impl HeliumReader {
    pub fn new(
        file_path: &str
    ) -> Self {
        Self {
            file_path: String::from(file_path),

            header: VdbHeader {
                file_version: 0,
                library_major: 0,
                library_minor: 0,
                has_grid_offsets: false,
                uuid: "".into()
            },
            file_metadata: Vec::new(),
            grid_descriptors: Vec::new(),
            grids: Vec::new(),
        }
    }

    pub fn read_file(
        &mut self
    ) -> Result<(), HeliumError> {

        let file_path = Path::new(&self.file_path);
        let file = File::open(file_path)?;
        let file_length = file.metadata()?.len();
        
        let mut reader = BufReader::new(file);

        println!("Reading VDB file at {}!", self.file_path);

        self.header = Self::read_header(&mut reader)?;
        self.file_metadata = Self::read_metadata(&mut reader)?;

        let grid_count = Self::read_grid_count(&mut reader)?;

        self.read_grid_descriptors(
            &mut reader, 
            grid_count, 
            file_length
        )?;

        self.grids = Self::read_grids(
            &mut reader, 
            &self.grid_descriptors, 
            file_length
        )?;

        println!("Done reading VDB file: ");
        // println!("{:#?}", self);

        Ok(())
    }

    fn read_header<R: Read>(reader: &mut R) -> Result<VdbHeader, HeliumError> {
        let magic = Self::read_u64_le(reader)?;

        if magic != OPENVDB_MAGIC {
            return Err(
                HeliumError::InvalidMagic{
                    expected: OPENVDB_MAGIC,
                    found: magic,
                }
            );
        }

        let file_version = Self::read_u32_le(reader)?;

        if !(MIN_SUPPORTED_VERSION..=MAX_SUPPORTED_VERSION).contains(&file_version) {
            return Err(HeliumError::UnsupportedVersion(file_version));
        }

        let library_major = Self::read_u32_le(reader)?;
        let library_minor = Self::read_u32_le(reader)?;

        let has_grid_offsets_raw = Self::read_u8(reader)?;
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

        Ok(VdbHeader{ 
            file_version, 
            library_major, 
            library_minor, 
            has_grid_offsets, 
            uuid
        })
    }

    fn read_metadata<R: Read>(reader: &mut R) -> Result<Vec<VdbMetadata>, HeliumError> {
        let metadata_count = Self::read_u32_le(reader)?;

        if metadata_count > MAX_METADATA_COUNT {
            return Err(
                HeliumError::InvalidMetadataCount(metadata_count)
            );
        }

        let mut metadata = Vec::with_capacity(metadata_count as usize);

        for _ in 0..metadata_count {
            let name = Self::read_string(reader)?;
            let type_name = Self::read_string(reader)?;

            let payload_size = Self::read_u32_le(reader)? as usize;

            if payload_size > MAX_STRING_LENGTH {
                return Err(HeliumError::MetadataPayloadTooLarge{
                    name, 
                    size: payload_size
                });
            }

            let mut payload = vec![0u8; payload_size];
            reader.read_exact(&mut payload)?;

            metadata.push(
                VdbMetadata {
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

    fn read_grid_count<R: Read>(reader: &mut R) -> Result<usize, HeliumError> {
        let grid_count = Self::read_i32_le(reader)?;

        if !(0..=MAX_GRID_COUNT).contains(&grid_count) {
            return Err(HeliumError::InvalidGridCount(grid_count));
        }

        Ok(grid_count as usize)
    }

    fn read_grid_descriptors<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        grid_count: usize,
        file_length: u64
    ) -> Result<(), HeliumError> {
        if !self.header.has_grid_offsets {
            return Err(HeliumError::GridOffsetsRequired);
        }

        self.grid_descriptors.clear();
        self.grid_descriptors.reserve(grid_count);

        for grid_index in 0..grid_count {
            let descriptor = Self::read_grid_descriptor(reader)?;

            Self::validate_grid_descriptor(
                grid_index, 
                &descriptor, 
                file_length
            )?;

            let next_descriptor_position = descriptor.end_position;

            self.grid_descriptors.push(descriptor);

            reader.seek(SeekFrom::Start(next_descriptor_position as u64))?;
        }

        Ok(())
    }

    fn read_grid_descriptor<R: Read>(reader: &mut R) -> Result<GridDescriptor, HeliumError> {
        let unique_name = Self::read_string(reader)?;
        let mut grid_type = Self::read_string(reader)?;
        let instance_parent_name = Self::read_string(reader)?;

        let grid_position = Self::read_i64_le(reader)?;
        let block_position = Self::read_i64_le(reader)?;
        let end_position = Self::read_i64_le(reader)?;

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
    ) -> Result<Vec<VdbGrid>, HeliumError> {
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
    ) -> Result<VdbGrid, HeliumError> {
        let grid_position = Self::checked_offset(descriptor.grid_position, "grid_position")?;
        let block_position = Self::checked_offset(descriptor.block_position, "block_position")?;
        let end_position = Self::checked_offset(descriptor.end_position, "end_position")?;

        if end_position > file_length {
            return Err(HeliumError::GridOffsetPastEnd{ 
                grid_index, 
                end_position, 
                file_length
            });
        }

        reader.seek(SeekFrom::Start(grid_position))?;

        let compression_raw = Self::read_u32_le(reader)?;
        let compression = CompressionFlags::from_raw(compression_raw);

        if compression.unknown_bits() != 0 {
            return Err(HeliumError::UnknownCompressionFlags {
                grid_index,
                flags: compression_raw,
            });
        }

        let metadata = Self::read_metadata(reader)?;
        let transform = Self::read_transform(reader)?;

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
            Self::read_byte_range(
                reader, 
                topology_start, 
                block_position
            )?
        };

        let raw_buffers = if is_instance {

            Vec::new()
        } else {
            Self::read_byte_range(
                reader, 
                block_position, 
                end_position
            )?
        };

        Ok(VdbGrid {
            descriptor: descriptor.clone(),
            compression,
            metadata,
            transform,
            raw_topology,
            raw_buffers
        })
    }

    fn checked_offset(
        offset: i64,
        field: &'static str,
    ) -> Result<u64, HeliumError> {
        u64::try_from(offset).map_err(|_| {
            HeliumError::NegativeStreamPosition {
                field,
                value: offset,
            }
        })
    }

    fn read_string<R: Read>(
        reader: &mut R,
    ) -> Result<String, HeliumError> {
        let length = Self::read_u32_le(reader)? as usize;

        if length > MAX_STRING_LENGTH {
            return Err(HeliumError::StringTooLarge(length));
        }

        let mut bytes = vec![0_u8; length];
        reader.read_exact(&mut bytes)?;

        String::from_utf8(bytes)
            .map_err(HeliumError::InvalidStringEncoding)
    }

    fn read_u8<R: Read>(reader: &mut R) -> Result<u8, HeliumError> {
        let mut bytes = [0_u8; 1];
        reader.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }

    fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, HeliumError> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i32_le<R: Read>(reader: &mut R) -> Result<i32, HeliumError> {
        let mut bytes = [0_u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_i64_le<R: Read>(reader: &mut R) -> Result<i64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_byte_range<R: Read + Seek>(
        reader: &mut R,
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>, HeliumError> {
        let length = end
            .checked_sub(start)
            .ok_or(HeliumError::InvalidByteRange { start, end })?;

        let length = usize::try_from(length)
            .map_err(|_| HeliumError::ByteRangeTooLarge { start, end })?;

        reader.seek(SeekFrom::Start(start))?;

        let mut bytes = vec![0_u8; length];
        reader.read_exact(&mut bytes)?;

        Ok(bytes)
    }

    fn read_f64_le<R: Read>(
        reader: &mut R,
    ) -> Result<f64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(f64::from_le_bytes(bytes))
    }

    fn read_vec3d<R: Read>(
        reader: &mut R,
    ) -> Result<[f64; 3], HeliumError> {
        Ok([
            Self::read_f64_le(reader)?,
            Self::read_f64_le(reader)?,
            Self::read_f64_le(reader)?,
        ])
    }

    fn read_mat4d<R: Read>(
        reader: &mut R,
    ) -> Result<[[f64; 4]; 4], HeliumError> {
        let mut matrix = [[0.0_f64; 4]; 4];

        for row in &mut matrix {
            for value in row {
                *value = Self::read_f64_le(reader)?;
            }
        }

        Ok(matrix)
    }

    fn read_transform<R: Read>(
        reader: &mut R,
    ) -> Result<VdbTransform, HeliumError> {
        let map_type = Self::read_string(reader)?;

        match map_type.as_str() {
            "AffineMap" => {
                let matrix = Self::read_mat4d(reader)?;

                Ok(VdbTransform::Affine {
                    matrix,
                })
            }

            "UnitaryMap" => {
                /*
                * UnitaryMap serializes its internal AffineMap, which
                * serializes one Mat4d.
                */
                let matrix = Self::read_mat4d(reader)?;

                Ok(VdbTransform::Unitary {
                    matrix,
                })
            }

            "ScaleMap" | "UniformScaleMap" => {
                /*
                * Despite the apparent redundancy, OpenVDB writes all
                * five cached Vec3d values.
                */
                let scale = Self::read_vec3d(reader)?;
                let voxel_size = Self::read_vec3d(reader)?;
                let inverse_scale = Self::read_vec3d(reader)?;
                let inverse_scale_squared = Self::read_vec3d(reader)?;
                let inverse_twice_scale = Self::read_vec3d(reader)?;

                Ok(VdbTransform::Scale {
                    map_type,
                    scale,
                    voxel_size,
                    inverse_scale,
                    inverse_scale_squared,
                    inverse_twice_scale,
                })
            }

            "TranslationMap" => {
                let translation = Self::read_vec3d(reader)?;

                Ok(VdbTransform::Translation {
                    translation,
                })
            }

            "ScaleTranslateMap"
            | "UniformScaleTranslateMap" => {
                let translation = Self::read_vec3d(reader)?;
                let scale = Self::read_vec3d(reader)?;
                let voxel_size = Self::read_vec3d(reader)?;
                let inverse_scale = Self::read_vec3d(reader)?;
                let inverse_scale_squared = Self::read_vec3d(reader)?;
                let inverse_twice_scale = Self::read_vec3d(reader)?;

                Ok(VdbTransform::ScaleTranslate {
                    map_type,
                    translation,
                    scale,
                    voxel_size,
                    inverse_scale,
                    inverse_scale_squared,
                    inverse_twice_scale,
                })
            }

            other => Err(
                HeliumError::UnsupportedTransform(
                    other.to_owned(),
                )
            ),
        }
    }
}