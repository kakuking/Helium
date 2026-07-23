use crate::reader::reader::HeliumReader;

pub mod reader;
pub mod tree;

fn main() -> Result<(), reader::HeliumError>{
    let _reader = match HeliumReader::open("./resources/smoke.vdb") {
        Ok(reader) => reader,
        Err(e) => return Err(e)
    };

    Ok(())
}
