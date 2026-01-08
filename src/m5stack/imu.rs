use esp_idf_hal::{i2c::I2cDriver, delay::FreeRtos};

const MPU6886_ADDR: u8 = 0x68;

pub struct Imu {
    initialized: bool,
}

impl Imu {
    /// Initialize MPU6886 IMU via shared I2C bus
    pub fn new(i2c: &mut I2cDriver) -> Self {
        println!("Initializing MPU6886 IMU...");
        
        // Wake up MPU6886
        i2c.write(MPU6886_ADDR, &[0x6B, 0x00], 1000).ok();
        FreeRtos::delay_ms(10);
        
        // Configure accelerometer and gyroscope
        i2c.write(MPU6886_ADDR, &[0x1C, 0x10], 1000).ok(); // Accel ±8g
        i2c.write(MPU6886_ADDR, &[0x1B, 0x18], 1000).ok(); // Gyro ±2000dps
        
        println!("IMU initialized");
        
        Self { initialized: true }
    }
    
    pub fn print_stats(&self, i2c: &mut I2cDriver) {
        if !self.initialized {
            println!("  IMU: Not initialized");
            return;
        }
        
        println!("Reading IMU data...");
        match Self::read_accel(i2c) {
            Some((ax, ay, az)) => println!("  Accel: x={:.2}, y={:.2}, z={:.2}", ax, ay, az),
            None => println!("  Accel: Read failed"),
        }
        
        match Self::read_gyro(i2c) {
            Some((gx, gy, gz)) => println!("  Gyro: x={:.2}, y={:.2}, z={:.2}", gx, gy, gz),
            None => println!("  Gyro: Read failed"),
        }
        
        match Self::read_temp(i2c) {
            Some(temp) => println!("  Temp: {:.1}C", temp),
            None => println!("  Temp: Read failed"),
        }
    }
    
    fn read_accel(i2c: &mut I2cDriver) -> Option<(f32, f32, f32)> {
        let mut data = [0u8; 6];
        i2c.write_read(MPU6886_ADDR, &[0x3B], &mut data, 1000).ok()?;
        
        let ax = i16::from_be_bytes([data[0], data[1]]) as f32 / 4096.0;
        let ay = i16::from_be_bytes([data[2], data[3]]) as f32 / 4096.0;
        let az = i16::from_be_bytes([data[4], data[5]]) as f32 / 4096.0;
        
        Some((ax, ay, az))
    }
    
    fn read_gyro(i2c: &mut I2cDriver) -> Option<(f32, f32, f32)> {
        let mut data = [0u8; 6];
        i2c.write_read(MPU6886_ADDR, &[0x43], &mut data, 1000).ok()?;
        
        let gx = i16::from_be_bytes([data[0], data[1]]) as f32 / 16.4;
        let gy = i16::from_be_bytes([data[2], data[3]]) as f32 / 16.4;
        let gz = i16::from_be_bytes([data[4], data[5]]) as f32 / 16.4;
        
        Some((gx, gy, gz))
    }
    
    fn read_temp(i2c: &mut I2cDriver) -> Option<f32> {
        let mut data = [0u8; 2];
        i2c.write_read(MPU6886_ADDR, &[0x41], &mut data, 1000).ok()?;
        
        let temp_raw = i16::from_be_bytes([data[0], data[1]]) as f32;
        Some((temp_raw / 326.8) + 25.0)
    }
}
