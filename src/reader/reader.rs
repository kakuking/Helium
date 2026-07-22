use std::{
    fs::File, 
    io::{BufReader, Read}, 
    path::Path
};

use crate::reader::{HeliumError, vdb_structs::{GridDescriptor, MAX_METADATA_COUNT, MAX_STRING_LENGTH, MAX_SUPPORTED_VERSION, MIN_SUPPORTED_VERSION, OPENVDB_MAGIC, VdbHeader, VdbMetadata}};

#[derive(Debug)]
pub struct HeliumReader {
    file_path: String,

    header: VdbHeader,
    metadata: Vec<VdbMetadata>,
    grids: Vec<GridDescriptor>
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
            metadata: Vec::new(),
            grids: Vec::new()
        }
    }

    pub fn read_file(
        &mut self
    ) -> Result<(), HeliumError> {

        let file_path = Path::new(&self.file_path);
        
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        println!("Reading VDB file at {}!", self.file_path);

        self.header = Self::read_header(&mut reader)?;
        self.metadata = Self::read_metadata(&mut reader)?;

        println!("Done reading VDB file: ");
        println!("{:#?}", self);

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

    fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, HeliumError> {
        let mut bytes = [0_u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }
}