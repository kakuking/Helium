use crate::reader::reader::HeliumReader;

pub mod reader;

fn main() {
    let mut reader = HeliumReader::new("./resources/smoke.vdb");

    let result = reader.read_file();

    match result {
        Ok(_) => println!("Successfully read file!"),
        Err(e) => eprintln!("Error: {e}")
    }
}
