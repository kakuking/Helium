pub const OPENVDB_MAGIC: u64 = 0x5644_4220;
pub const MIN_SUPPORTED_VERSION: u32 = 222;
pub const MAX_SUPPORTED_VERSION: u32 = 225;
pub const MAX_STRING_LENGTH: usize = 16 * 1024 * 1024;
pub const MAX_METADATA_COUNT: u32 = 1_000_000;
pub const MAX_GRID_COUNT: i32 = 1_000_000;

pub const UPPER_NODE_LOG2_DIM: usize = 5;
pub const LOWER_NODE_LOG2_DIM: usize = 4;
pub const LEAF_NODE_LOG2_DIM: usize = 3;

pub const UPPER_NODE_DIM: usize =
    1 << UPPER_NODE_LOG2_DIM;

pub const LOWER_NODE_DIM: usize =
    1 << LOWER_NODE_LOG2_DIM;

pub const LEAF_NODE_DIM: usize =
    1 << LEAF_NODE_LOG2_DIM;

pub const UPPER_NODE_SIZE: usize =
    UPPER_NODE_DIM
    * UPPER_NODE_DIM
    * UPPER_NODE_DIM;

pub const LOWER_NODE_SIZE: usize =
    LOWER_NODE_DIM
    * LOWER_NODE_DIM
    * LOWER_NODE_DIM;

pub const LEAF_NODE_SIZE: usize =
    LEAF_NODE_DIM
    * LEAF_NODE_DIM
    * LEAF_NODE_DIM;

pub const MAX_ROOT_TILE_COUNT: usize = 10_000_000;
pub const MAX_ROOT_CHILD_COUNT: usize = 10_000_000;