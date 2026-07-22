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

    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GridDescriptor {
    pub unique_name: String,
    pub grid_type: String,
    pub instance_parent_name: String,

    pub grid_position: i64,
    pub block_position: i64,
    pub end_position: i64,

    pub save_float_as_half: bool,
}

#[derive(Debug)]
pub struct VdbGrid {
    pub descriptor: GridDescriptor,

    pub compression: CompressionFlags,
    pub metadata: Vec<VdbMetadata>,
    pub transform: VdbTransform,

    pub raw_topology: Vec<u8>,

    pub raw_buffers: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompressionFlags {
    pub raw: u32,
    pub zip: bool,
    pub active_mask: bool,
    pub blosc: bool,
}

impl CompressionFlags {
    pub const ZIP: u32 = 0x1;
    pub const ACTIVE_MASK: u32 = 0x2;
    pub const BLOSC: u32 = 0x4;

    pub fn from_raw(raw: u32) -> Self {
        Self {
            raw,
            zip: raw & Self::ZIP != 0,
            active_mask: raw & Self::ACTIVE_MASK != 0,
            blosc: raw & Self::BLOSC != 0,
        }
    }

    pub fn unknown_bits(self) -> u32 {
        self.raw & !(Self::ZIP | Self::ACTIVE_MASK | Self::BLOSC)
    }
}

#[derive(Debug)]
pub enum VdbTransform {
    Affine {
        matrix: [[f64; 4]; 4],
    },

    Scale {
        map_type: String,
        scale: [f64; 3],
        voxel_size: [f64; 3],
        inverse_scale: [f64; 3],
        inverse_scale_squared: [f64; 3],
        inverse_twice_scale: [f64; 3],
    },

    Translation {
        translation: [f64; 3],
    },

    ScaleTranslate {
        map_type: String,
        translation: [f64; 3],
        scale: [f64; 3],
        voxel_size: [f64; 3],
        inverse_scale: [f64; 3],
        inverse_scale_squared: [f64; 3],
        inverse_twice_scale: [f64; 3],
    },

    Unitary {
        matrix: [[f64; 4]; 4],
    },
}
