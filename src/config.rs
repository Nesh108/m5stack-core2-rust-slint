// Display configuration
pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;
pub const SPI_BAUDRATE_HZ: u32 = 26_000_000;

// I2C addresses
pub const AXP192_ADDR: u8 = 0x34;
pub const FT6336_ADDR: u8 = 0x38;

// I2C configuration
pub const I2C_BAUDRATE_HZ: u32 = 400_000;

// Rendering configuration
pub const TARGET_FPS: u32 = 30;
pub const FRAME_TIME_MS: u32 = 1000 / TARGET_FPS;

// Display buffer transfer chunk size
pub const TRANSFER_CHUNK_SIZE: usize = 4096;
