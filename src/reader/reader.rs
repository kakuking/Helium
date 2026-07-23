use std::{
    fmt::Debug, 
    fs::File, 
    io::BufReader, 
    path::Path
};

use crate::reader::{
    HeliumError, 
    grids_reader::Grids, 
    header::Header, 
    metadata::Metadata
};

#[derive(Debug)]
pub struct HeliumReader {
    file_path: String,

    header: Header,
    file_metadata: Vec<Metadata>,
    grids: Grids
}

impl HeliumReader {
    pub fn new(
        file_path: &str
    ) -> Self {
        Self {
            file_path: String::from(file_path),

            header: Header::new(),
            file_metadata: Vec::new(),
            grids: Grids::new(),
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

        self.header = Header::read(&mut reader)?;
        self.file_metadata = Metadata::read(&mut reader)?;

        self.grids = Grids::read(
            &mut reader, 
            &self.header, 
            file_length
        )?;

        println!("Done reading VDB file: ");
        println!("{:#?}", self);

        Ok(())
    }
}