pub const VERSION: u8 = 1;

pub const TIMESTAMP_BYTES: usize = 6;
pub const VERSION_BYTES: usize = 1;
pub const TYPE_BYTES: usize = 1;
pub const ENTROPY_BYTES: usize = 16;

pub const TOTAL_BYTES: usize =
  TIMESTAMP_BYTES + VERSION_BYTES + TYPE_BYTES + ENTROPY_BYTES;
pub const ENCODED_LENGTH: usize = 39;
