use esp_idf_hal::{
    gpio::{PinDriver, Output, Gpio2},
    i2c::I2cDriver,
};

const NS4168_ADDR: u8 = 0x4C;

pub struct Speaker<'d> {
    enable_pin: PinDriver<'d, Gpio2, Output>,
}

impl<'d> Speaker<'d> {
    /// Initialize NS4168 speaker amplifier for M5Stack Core2
    pub fn new(mut enable_pin: PinDriver<'d, Gpio2, Output>, i2c: &mut I2cDriver) -> Self {
        println!("Initializing NS4168 speaker amplifier...");
        
        // Try NS4168 I2C configuration
        i2c.write(NS4168_ADDR, &[0x00, 0x01], 1000).ok();
        
        // Start with amplifier disabled
        enable_pin.set_low().ok();
        
        println!("Speaker ready (GPIO2 enable control)");
        
        Self { enable_pin }
    }
    
    /// Enable speaker
    pub fn start_beep(&mut self) {
        println!("  Enabling NS4168 amplifier");
        self.enable_pin.set_high().ok();
    }
    
    /// Disable speaker
    pub fn stop_beep(&mut self) {
        println!("  Disabling NS4168 amplifier");
        self.enable_pin.set_low().ok();
    }
}
