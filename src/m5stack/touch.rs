// FT6336 touch controller for M5Stack Core2

use esp_idf_hal::i2c::I2cDriver;
use crate::m5stack::config::FT6336_ADDR;

/// Read touch coordinates from FT6336 touch controller
pub fn read_touch(i2c: &mut I2cDriver) -> Option<(u16, u16)> {
    let mut buf = [0u8];
    
    if i2c.write_read(FT6336_ADDR, &[0x02], &mut buf, 1000).is_err() || buf[0] == 0 {
        return None;
    }
    
    let mut data = [0u8; 4];
    if i2c.write_read(FT6336_ADDR, &[0x03], &mut data, 1000).is_err() {
        return None;
    }
    
    let x = (((data[0] & 0x0F) as u16) << 8) | (data[1] as u16);
    let y = (((data[2] & 0x0F) as u16) << 8) | (data[3] as u16);
    
    Some((x, y))
}

/// Check which button zone was touched on M5Stack Core2 display
/// The bottom of the screen has three buttons: BtnA, BtnB, BtnC
pub fn check_button_zone(x: u16, y: u16) -> Option<&'static str> {
    if y < 210 {
        return None;
    }
    
    if x < 107 {
        Some("BtnA")
    } else if x < 214 {
        Some("BtnB")
    } else {
        Some("BtnC")
    }
}
