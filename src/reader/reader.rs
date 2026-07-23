use std::{
    fmt::Debug, 
    fs::File, 
    io::{BufReader, Read, Seek}, 
    path::Path
};

use crate::reader::{
    HeliumError, 
    grids::Grids, 
    header::Header, 
    metadata::Metadata
};

#[derive(Debug)]
pub struct HeliumReader<R: Read + Seek> {
    pub reader: R,
    pub file_path: String,
    pub header: Header,
    pub file_metadata: Vec<Metadata>,
    pub grids: Grids
}

impl HeliumReader<BufReader<File>> {
    pub fn open(
        file_path_str: &str
    ) -> Result<Self, HeliumError> {
        let file_path = Path::new(&file_path_str);
        let file = File::open(file_path)?;
        let file_length = file.metadata()?.len();
        
        let mut reader = BufReader::new(file);

        println!("Reading VDB file at {}!", file_path_str);

        let header = Header::read(&mut reader)?;
        let file_metadata = Metadata::read(&mut reader)?;

        let grids = Grids::read(
            &mut reader, 
            &header, 
            file_length
        )?;

        println!("Done reading VDB file: ");

        let helium_reader = HeliumReader {
            reader,
            file_path: file_path_str.into(),
            header,
            file_metadata,
            grids
        };

        println!("{:#?}", helium_reader);

        Ok(helium_reader)
    }
}