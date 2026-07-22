use std::{
    fs::File, 
    io::{BufReader, Read}, 
    path::Path
};

use crate::reader::HeliumError;

#[derive(Debug)]
pub struct HeliumReader {
    file_path: String,
    openvdb_file_version: u32,
    openvdb_major_version: u8,
    openvdb_minor_version: u8,
}

impl HeliumReader {
    pub fn new(
        file_path: &str
    ) -> Self {
        Self {
            file_path: String::from(file_path),

            openvdb_file_version: 0,
            openvdb_major_version: 0,
            openvdb_minor_version: 0,
        }
    }

    pub fn read_file(
        &mut self
    ) -> Result<(), HeliumError> {

        let file_path = Path::new(&self.file_path);
        
        let file = File::open(file_path)?;
        let file_reader = BufReader::new(file);

        println!("Reading byte by byte!");

        let bytes = file_reader.bytes();

        for (idx, byte_result) in bytes.enumerate() {
            if idx > 10 {
                break;
            }

            let byte = byte_result?;

            // Check if magic numbers match
            if idx < 8 &&
            (match idx {
                0 => byte != 32,
                1 => byte != 66,
                2 => byte != 68,
                3 => byte != 86,
                4 => byte != 0,
                5 => byte != 0,
                6 => byte != 0,
                7 => byte != 0,
                _ => true
            }) {
                return Err(HeliumError::InvalidMagic(idx));
            }

            // Setting OpenVDB file version
            if idx == 8 {
                self.openvdb_file_version = byte as u32;
            }

            // Setting OpenVDB major and minor version
            if idx == 9 {
                self.openvdb_major_version = byte;
            }

            if idx == 10 {
                self.openvdb_minor_version = byte;
            }
        }

        self.print();

        Ok(())
    }

    pub fn print(&self) {
        let reader_string = format!("HeliumReader[ \n\
            \tfile_path: {}, \n\
            \tOpenVDB File Version: {}, \n\
            \tOpenVDB Major Version: {}, \n\
            \tOpenVDB Minor Version: {}, \n\
            ]", 
            self.file_path, 
            self.openvdb_file_version, 
            self.openvdb_major_version, 
            self.openvdb_minor_version
        );

        println!("{}", reader_string);
    }
}