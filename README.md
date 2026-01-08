# M5Stack Core2 with Slint UI and Rust

A demonstration of using the Slint UI framework on M5Stack Core2 hardware, entirely in Rust.

## Features

- ✅ **Display:** ILI9342C TFT (320x240) via SPI with Slint UI framework
- ✅ **Touch:** FT6336 capacitive touch controller
- ✅ **IMU:** MPU6886 6-axis motion sensor
- ✅ **Power:** AXP192 power management
- ✅ **LED:** Onboard LED control via AXP192
- ✅ **Audio:** I2S audio with NS4168 amplifier (musical scale playback)

## Button Functions

- **BtnA (Left)** - Play musical scale (C-D-E-F-G-A-B-C)
- **BtnB (Middle)** - Toggle LED
- **BtnC (Right)** - Show IMU & Battery stats

## Project Structure

```
src/
├── main.rs              - Clean application entry point
├── slint_platform.rs    - Slint platform integration
├── m5stack/             - M5Stack Core2 hardware abstraction
│   ├── mod.rs          - Module exports
│   ├── config.rs       - Hardware configuration constants
│   ├── display.rs      - ILI9342C display controller
│   ├── touch.rs        - FT6336 touch controller + button zones
│   ├── power.rs        - AXP192 power management + LED
│   ├── imu.rs          - MPU6886 IMU sensor
│   └── audio.rs        - I2S audio with NS4168 amplifier
└── ui/
    └── hello.slint     - Slint UI definition
```

## Hardware Configuration

### M5Stack Core2 Pins

**I2C (Sensors & Power):**
- GPIO21: SDA
- GPIO22: SCL
- Devices: AXP192 (0x34), FT6336 (0x38), MPU6886 (0x68), NS4168 (0x4C)

**SPI (Display):**
- GPIO18: SCK
- GPIO23: MOSI
- GPIO38: MISO
- GPIO5: CS
- GPIO15: DC

**Audio I2S:**
- GPIO12: BCK (Bit Clock)
- GPIO0: WS (Word Select)
- GPIO2: DATA (⚠️ shared with display - used temporarily)

**Power & Peripherals:**
- AXP192: Power management, LED control, battery monitoring
- GPIO2: Audio amplifier enable (conflicts with display)

## Audio Implementation

The audio system (`src/m5stack/audio.rs`) is based on the Arduino M5Unified Speaker library:

### Key Components:
1. **I2S Hardware** - ESP32 I2S peripheral for digital audio
2. **AXP192 GPIO2** - Critical power enable for NS4168 amplifier (register 0x93 = 0x06)
3. **NS4168 Amplifier** - I2C-controlled Class D audio amplifier
4. **Tone Generation** - Uses 16-sample sine wave from M5Unified

### GPIO2 Conflict Solution:
The audio module is created **on-demand** when BtnA is pressed:
1. Audio module instantiated (takes GPIO2)
2. Musical scale plays
3. Audio module dropped (GPIO2 released to display)

This causes brief display corruption during playback but allows both features to work.

## Building and Flashing

```bash
# Build
cargo build

# Flash and monitor
cargo espflash flash --monitor

# Release build
cargo build --release
```

## Dependencies

- `esp-idf-svc` 0.50 - ESP-IDF service layer
- `esp-idf-hal` 0.45 - ESP32 HAL
- `esp-idf-sys` 0.36 - ESP-IDF bindings
- `slint` 1.5 - UI framework
- `embedded-graphics` 0.8
- `display-interface` 0.5
- `mipidsi` 0.8

## M5Stack Libraries

The `libraries/` directory contains M5Unified Arduino libraries used as reference:
- M5Unified - Core M5Stack functionality
- M5GFX - Graphics library
- M5Utility - Utility functions

These are not compiled but used to understand the low-level hardware initialization.

## Audio Details

Based on M5Unified `Speaker_Class.cpp`, the critical discovery was:
- Arduino: `M5.Power.Axp192.setGPIO2(true)` enables speaker
- Implementation: Write `0x06` to AXP192 register `0x93`

Without this single I2C command, I2S sends data but no sound is produced.

## License

MIT

## Hardware

M5Stack Core2:
- ESP32 dual-core @ 240MHz
- 16MB Flash
- 8MB PSRAM
- 320x240 ILI9342C TFT
- FT6336 capacitive touch
- MPU6886 IMU
- AXP192 power management
- NS4168 audio amplifier
