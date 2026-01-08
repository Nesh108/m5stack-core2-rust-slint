// M5Stack Core2 hardware abstraction module

pub mod config;
pub mod display;
pub mod power;
pub mod imu;
pub mod audio;
pub mod touch;

// Re-export commonly used items
pub use config::*;
pub use display::*;
pub use power::*;
pub use touch::*;
pub use imu::Imu;
pub use audio::Audio;
