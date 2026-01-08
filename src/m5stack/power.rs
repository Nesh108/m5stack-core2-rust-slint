use esp_idf_hal::{i2c::I2cDriver, delay::FreeRtos};
use crate::m5stack::config::AXP192_ADDR;

const AXP192_GPIO12_SIGNAL: u8 = 0x94;
const AXP192_GPIO1_CONTROL: u8 = 0x92;

/// Initialize AXP192 power management for M5Stack Core2
pub fn init_power(i2c: &mut I2cDriver) {
    i2c.write(AXP192_ADDR, &[0x28, 0xCC], 1000).ok();
    i2c.write(AXP192_ADDR, &[0x27, 0xDC], 1000).ok();
    i2c.write(AXP192_ADDR, &[0x91, 0xF0], 1000).ok();
    i2c.write(AXP192_ADDR, &[0x90, 0x02], 1000).ok();
    i2c.write(AXP192_ADDR, &[0x96, 0x02], 1000).ok();
    FreeRtos::delay_ms(200);
}

/// Initialize LED control via AXP192 GPIO1
pub fn init_led(i2c: &mut I2cDriver) {
    // Set GPIO1 to NMOS open-drain output mode
    i2c.write(AXP192_ADDR, &[AXP192_GPIO1_CONTROL, 0x00], 1000).ok();
}

/// Control LED state
/// The LED is connected to AXP192 GPIO1 in open-drain configuration
/// Low = LED ON, High/Float = LED OFF
pub fn set_led(i2c: &mut I2cDriver, on: bool) {
    let mut val = [0u8];
    i2c.write_read(AXP192_ADDR, &[AXP192_GPIO12_SIGNAL], &mut val, 1000).ok();
    
    if on {
        // Clear bit 1 (GPIO1 low = LED ON)
        val[0] &= !0x02;
    } else {
        // Set bit 1 (GPIO1 high = LED OFF)
        val[0] |= 0x02;
    }
    
    i2c.write(AXP192_ADDR, &[AXP192_GPIO12_SIGNAL, val[0]], 1000).ok();
}

/// Read battery voltage from AXP192
pub fn read_battery_voltage(i2c: &mut I2cDriver) -> Option<f32> {
    let mut data = [0u8; 2];
    i2c.write_read(AXP192_ADDR, &[0x78], &mut data, 1000).ok()?;
    
    let raw = ((data[0] as u16) << 4) | ((data[1] as u16) >> 4);
    let volts = (raw as f32) * 1.1 / 1000.0;
    
    Some(volts)
}

/// Calculate battery percentage from voltage
/// Typical Li-ion: 3.0V=0%, 4.2V=100%
pub fn battery_percentage(volts: f32) -> u8 {
    let percent = ((volts - 3.0) / (4.2 - 3.0) * 100.0).clamp(0.0, 100.0);
    percent as u8
}
