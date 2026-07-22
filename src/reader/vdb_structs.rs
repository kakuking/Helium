pub const OPENVDB_MAGIC: u64 = 0x5644_4220;
pub const MIN_SUPPORTED_VERSION: u32 = 222;
pub const MAX_SUPPORTED_VERSION: u32 = 225;
pub const MAX_STRING_LENGTH: usize = 16 * 1024 * 1024;
pub const MAX_METADATA_COUNT: u32 = 1_000_000;
pub const MAX_GRID_COUNT: i32 = 1_000_000;

#[derive(Debug)]
pub struct VdbHeader {
    pub file_version: u32,
    pub library_major: u32,
    pub library_minor: u32,
    pub has_grid_offsets: bool,
    pub uuid: String,
}

#[derive(Debug)]
pub struct VdbMetadata {
    pub name: String,
    pub type_name: String,

    /// Raw metadata payload.
    ///
    /// We can preserve unknown metadata types because OpenVDB stores
    /// the payload size before the payload itself.
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct GridDescriptor {
    pub unique_name: String, // Internal unique name. This may differ from the user-facing grid name.

    pub grid_type: String, // Runtime grid/tree type, (`Tree_float_5_4_3`), etc

    pub instance_parent_name: String, // Non-empty when this grid instances another grid's tree.

    pub grid_position: u64, // Byte position at which the grid body begins.

    pub block_position: u64, // Byte position at which separately stored grid buffers begin.

    pub end_position: u64, // Byte position immediately following this grid.

    pub save_float_as_half: bool, // OpenVDB encodes half-float storage using a suffix on the type name.
}