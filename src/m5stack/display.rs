// ILI9342C display controller for M5Stack Core2

use esp_idf_hal::delay::FreeRtos;

/// Initialize ILI9342C display controller with proper color configuration
pub fn init_display<DI: display_interface::WriteOnlyDataCommand>(di: &mut DI) {
    use display_interface::DataFormat;
    
    di.send_commands(DataFormat::U8(&[0x01])).ok();
    FreeRtos::delay_ms(120);
    di.send_commands(DataFormat::U8(&[0x11])).ok();
    FreeRtos::delay_ms(120);
    
    di.send_commands(DataFormat::U8(&[0x36])).ok();
    di.send_data(DataFormat::U8(&[0x08])).ok();
    
    di.send_commands(DataFormat::U8(&[0x3A])).ok();
    di.send_data(DataFormat::U8(&[0x55])).ok();
    di.send_commands(DataFormat::U8(&[0x21])).ok();
    di.send_commands(DataFormat::U8(&[0x29])).ok();
    FreeRtos::delay_ms(10);
}

/// Transfer Slint render buffer to display via SPI
pub fn transfer_buffer_to_display<DI: display_interface::WriteOnlyDataCommand>(
    di: &mut DI,
    buffer: &[slint::platform::software_renderer::Rgb565Pixel],
    chunk_size: usize,
) {
    use display_interface::DataFormat;
    use alloc::vec::Vec;
    
    di.send_commands(DataFormat::U8(&[0x2A])).ok();
    di.send_data(DataFormat::U8(&[0, 0, 0x01, 0x3F])).ok();
    di.send_commands(DataFormat::U8(&[0x2B])).ok();
    di.send_data(DataFormat::U8(&[0, 0, 0x00, 0xEF])).ok();
    di.send_commands(DataFormat::U8(&[0x2C])).ok();
    
    for chunk_start in (0..buffer.len()).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(buffer.len());
        let chunk = &buffer[chunk_start..chunk_end];
        
        let mut chunk_bytes = Vec::with_capacity(chunk.len() * 2);
        for pixel in chunk {
            let rgb = pixel.0;
            chunk_bytes.push((rgb >> 8) as u8);
            chunk_bytes.push((rgb & 0xFF) as u8);
        }
        
        di.send_data(DataFormat::U8(&chunk_bytes)).ok();
    }
}
