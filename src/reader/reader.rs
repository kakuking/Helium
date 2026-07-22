use std::{
    fs::File, 
    io::{BufReader, Read}, 
    path::Path
};

use crate::reader::{HeliumError, OPENVDB_MAGIC};

#[derive(Debug)]
pub struct HeliumReader {
    file_path: String,

    header: VdbHeader,
}

#[derive(Debug)]
pub struct VdbHeader {
    pub file_version: u32,
    pub library_major: u32,
    pub library_minor: u32,
    pub has_grid_offsets: bool,
    pub uuid: String,
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
            }
        }
    }

    pub fn read_file(
        &mut self
    ) -> Result<(), HeliumError> {

        let file_path = Path::new(&self.file_path);
        
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        println!("Reading VDB file at {}!", self.file_path);

        self.read_header(&mut reader)?;

        println!("Done reading VDB file: ");
        self.print();

        Ok(())
    }

    fn read_header<R: Read>(&mut self, reader: &mut R) -> Result<(), HeliumError> {
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

        if !(221..=225).contains(&file_version) {
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

        self.header = VdbHeader{ 
            file_version, 
            library_major, 
            library_minor, 
            has_grid_offsets, 
            uuid
        };

        Ok(())
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

    pub fn print(&self) {
        let reader_string = format!("HeliumReader[ \n\
            \tfile_path: {}, \n\
            \tOpenVDB File Version: {}, \n\
            \tOpenVDB Major Version: {}, \n\
            \tOpenVDB Minor Version: {}, \n\
            \tHas Grid Offsets: {}, \n\
            \tUUID: {}, \n\
            ]", 
            self.file_path, 
            self.header.file_version, 
            self.header.library_major, 
            self.header.library_minor,
            self.header.has_grid_offsets,
            self.header.uuid
        );

        println!("{}", reader_string);
    }
}