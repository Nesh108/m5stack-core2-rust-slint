// Audio playback for M5Stack Core2
// Key: AXP192 GPIO2 must be enabled for speaker amplifier

extern crate alloc;
use alloc::vec::Vec;

use esp_idf_hal::{
    gpio::{PinDriver, Output, Gpio2},
    i2c::I2cDriver,
    delay::FreeRtos,
};
use esp_idf_sys as sys;

const NS4168_ADDR: u8 = 0x4C;
const AXP192_ADDR: u8 = 0x34;
const SAMPLE_RATE: u32 = 44100;

// Default sine wave for tone generation (from M5Unified)
const DEFAULT_TONE_WAV: [u8; 16] = [
    177, 219, 246, 255, 246, 219, 177, 128, 
    79, 37, 10, 1, 10, 37, 79, 128
];

pub struct Audio<'d> {
    i2s_port: sys::i2s_port_t,
    enable_pin: PinDriver<'d, Gpio2, Output>,
}

impl<'d> Audio<'d> {
    pub fn new(enable_pin: PinDriver<'d, Gpio2, Output>, i2c: &mut I2cDriver) -> Result<Self, sys::EspError> {
        let i2s_port = sys::i2s_port_t_I2S_NUM_0;
        
        // CRITICAL: Enable speaker via AXP192 GPIO2
        // Register 0x93: 0x06=floating/HIGH(enable), 0x05=LOW(disable)
        i2c.write(AXP192_ADDR, &[0x93, 0x06], 1000).ok();
        FreeRtos::delay_ms(50);
        
        // Configure NS4168 amplifier
        i2c.write(NS4168_ADDR, &[0x00, 0x01], 1000).ok();
        FreeRtos::delay_ms(10);
        
        // I2S configuration
        let mut cfg = sys::i2s_config_t::default();
        cfg.mode = (sys::i2s_mode_t_I2S_MODE_MASTER | sys::i2s_mode_t_I2S_MODE_TX) as sys::i2s_mode_t;
        cfg.sample_rate = SAMPLE_RATE;
        cfg.bits_per_sample = sys::i2s_bits_per_sample_t_I2S_BITS_PER_SAMPLE_16BIT;
        cfg.channel_format = sys::i2s_channel_fmt_t_I2S_CHANNEL_FMT_RIGHT_LEFT;
        cfg.communication_format = sys::i2s_comm_format_t_I2S_COMM_FORMAT_STAND_I2S;
        cfg.intr_alloc_flags = sys::ESP_INTR_FLAG_LEVEL1 as i32;
        cfg.__bindgen_anon_1.dma_desc_num = 8;
        cfg.__bindgen_anon_2.dma_frame_num = 256;
        cfg.tx_desc_auto_clear = true;
        
        let pins = sys::i2s_pin_config_t {
            bck_io_num: 12,
            ws_io_num: 0,
            data_out_num: 2,
            data_in_num: -1,
            ..Default::default()
        };
        
        unsafe {
            sys::esp!(sys::i2s_driver_install(i2s_port, &cfg, 0, core::ptr::null_mut()))?;
            sys::esp!(sys::i2s_set_pin(i2s_port, &pins))?;
        }
        
        Ok(Self { i2s_port, enable_pin })
    }
    
    fn enable(&mut self) {
        self.enable_pin.set_high().ok();
    }
    
    fn disable(&mut self) {
        self.enable_pin.set_low().ok();
    }
    
    /// Play a tone
    pub fn tone(&mut self, freq: f32, duration_ms: u32) -> Result<(), sys::EspError> {
        self.enable();
        
        let wave_data = &DEFAULT_TONE_WAV;
        let wave_len = wave_data.len();
        let total_samples = (SAMPLE_RATE as f32 * duration_ms as f32 / 1000.0) as usize;
        let step = freq * (wave_len as f32) / (SAMPLE_RATE as f32);
        
        let mut buffer = Vec::with_capacity(total_samples * 4);
        let mut phase = 0.0f32;
        
        for _ in 0..total_samples {
            let index = (phase as usize) % wave_len;
            let sample_u8 = wave_data[index];
            let sample_i32 = (sample_u8 as i32 - 128) * 200;
            let sample = sample_i32.clamp(-32768, 32767) as i16;
            let bytes = sample.to_le_bytes();
            buffer.extend_from_slice(&bytes);
            buffer.extend_from_slice(&bytes);
            phase += step;
            if phase >= wave_len as f32 { phase -= wave_len as f32; }
        }
        
        let mut bytes_written: usize = 0;
        unsafe {
            sys::esp!(sys::i2s_write(
                self.i2s_port,
                buffer.as_ptr() as *const _,
                buffer.len(),
                &mut bytes_written,
                1000,
            ))?;
        }
        
        FreeRtos::delay_ms(duration_ms);
        self.disable();
        Ok(())
    }
    
    /// Play musical scale (C D E F G A B C)
    pub fn play_scale(&mut self) {
        let notes = [
            (262.0, "C4"), (294.0, "D4"), (330.0, "E4"), (349.0, "F4"),
            (392.0, "G4"), (440.0, "A4"), (494.0, "B4"), (523.0, "C5"),
        ];
        
        for (freq, _) in notes.iter() {
            self.tone(*freq, 150).ok();
            FreeRtos::delay_ms(50);
        }
    }
}

impl<'d> Drop for Audio<'d> {
    fn drop(&mut self) {
        unsafe { sys::i2s_driver_uninstall(self.i2s_port); }
    }
}
