use esp_idf_hal::{
    gpio::{PinDriver, Output, Gpio2},
    i2c::I2cDriver,
    delay::FreeRtos,
};
use esp_idf_sys::{self as sys, esp};
use alloc::vec::Vec;

const NS4168_ADDR: u8 = 0x4C;
const SAMPLE_RATE: u32 = 44100;

pub struct Audio<'d> {
    enable_pin: PinDriver<'d, Gpio2, Output>,
    i2s_handle: sys::i2s_chan_handle_t,
}

impl<'d> Audio<'d> {
    /// Initialize audio system for M5Stack Core2 with I2S
    pub fn new(
        mut enable_pin: PinDriver<'d, Gpio2, Output>,
        i2c: &mut I2cDriver,
    ) -> Self {
        println!("Initializing NS4168 audio system with I2S...");
        
        // Configure NS4168 amplifier via I2C
        i2c.write(NS4168_ADDR, &[0x00, 0x01], 1000).ok();
        
        // Start with amplifier disabled
        enable_pin.set_low().ok();
        
        // Configure I2S channel with proper DMA buffers
        let mut chan_cfg = sys::i2s_chan_config_t::default();
        chan_cfg.dma_desc_num = 6;
        chan_cfg.dma_frame_num = 240;
        chan_cfg.auto_clear = true;
        
        let mut tx_handle: sys::i2s_chan_handle_t = core::ptr::null_mut();
        
        let i2s_initialized = unsafe {
            // Create I2S channel
            if esp!(sys::i2s_new_channel(&chan_cfg, &mut tx_handle as *mut _, core::ptr::null_mut())).is_err() {
                false
            } else {
                // Configure I2S standard mode with GPIO pins
                let mut std_cfg = sys::i2s_std_config_t::default();
                std_cfg.clk_cfg.sample_rate_hz = SAMPLE_RATE;
                std_cfg.gpio_cfg.bclk = 12;  // BCK
                std_cfg.gpio_cfg.ws = 0;     // WS/LRCK
                std_cfg.gpio_cfg.dout = 2;   // DOUT
                std_cfg.gpio_cfg.din = -1;
                
                esp!(sys::i2s_channel_init_std_mode(tx_handle, &std_cfg)).is_ok()
                    && esp!(sys::i2s_channel_enable(tx_handle)).is_ok()
            }
        };
        
        if !i2s_initialized && !tx_handle.is_null() {
            unsafe { sys::i2s_del_channel(tx_handle); }
        }
        
        if i2s_initialized {
            println!("Audio system ready (I2S + NS4168)");
        } else {
            println!("Warning: I2S initialization failed");
        }
        
        Self {
            enable_pin,
            i2s_handle: tx_handle,
        }
    }
    
    /// Play a tone using I2S
    pub fn play_tone(&mut self, frequency_hz: u32, duration_ms: u32) {
        if self.i2s_handle.is_null() {
            println!("  I2S not initialized");
            return;
        }
        
        println!("  Playing tone {}Hz for {}ms", frequency_hz, duration_ms);
        
        // Enable amplifier
        self.enable_pin.set_high().ok();
        
        // Generate sine wave samples
        let num_samples = (SAMPLE_RATE * duration_ms / 1000) as usize;
        let mut samples: Vec<i16> = Vec::with_capacity(num_samples);
        
        let amplitude: i16 = 10000; // Volume
        let angular_freq = 2.0 * core::f32::consts::PI * frequency_hz as f32 / SAMPLE_RATE as f32;
        
        for i in 0..num_samples {
            let sample = (amplitude as f32 * (angular_freq * i as f32).sin()) as i16;
            samples.push(sample);
        }
        
        // Convert to bytes
        let bytes: Vec<u8> = samples.iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();
        
        // Write to I2S channel
        let mut bytes_written: usize = 0;
        unsafe {
            sys::i2s_channel_write(
                self.i2s_handle,
                bytes.as_ptr() as *const _,
                bytes.len(),
                &mut bytes_written as *mut _,
                1000,
            );
        }
        
        // Wait for playback
        FreeRtos::delay_ms(duration_ms + 10);
        
        // Disable amplifier
        self.enable_pin.set_low().ok();
    }
    
    /// Play startup jingle
    pub fn play_startup(&mut self) {
        println!("Playing startup jingle...");
        self.play_tone(2000, 100);
        FreeRtos::delay_ms(50);
        self.play_tone(1000, 100);
    }
    
    /// Stop audio
    pub fn stop(&mut self) {
        self.enable_pin.set_low().ok();
    }
}

impl<'d> Drop for Audio<'d> {
    fn drop(&mut self) {
        self.stop();
        if !self.i2s_handle.is_null() {
            unsafe {
                sys::i2s_channel_disable(self.i2s_handle);
                sys::i2s_del_channel(self.i2s_handle);
            }
        }
    }
}
